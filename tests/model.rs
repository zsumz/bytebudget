//! Exhaustive small-domain transition and command-sequence models.

use bytebudget::{ByteBudget, ByteCount};

fn count(bytes: u64) -> ByteCount {
    ByteCount::new(bytes)
}

fn observable(budget: &ByteBudget) -> (u64, u64, u64, bool) {
    (
        budget.limit().get(),
        budget.used().get(),
        budget.available().get(),
        budget.is_exhausted(),
    )
}

fn assert_invariant(budget: &ByteBudget, expected_used: u64) {
    let limit = budget.limit().get();
    let used = budget.used().get();
    let available = budget.available().get();
    assert!(used <= limit);
    assert_eq!(used, expected_used);
    assert_eq!(available, limit - used);
    assert_eq!(used + available, limit);
    assert_eq!(budget.is_exhausted(), used == limit);
}

#[test]
fn every_small_domain_reservation_matches_the_integer_model() {
    for limit in 0..=32 {
        for used in 0..=limit {
            for amount in 0..=33 {
                let mut budget = ByteBudget::new(count(limit));
                budget.try_reserve(count(used)).unwrap();
                let before = observable(&budget);
                let available = limit - used;

                match budget.try_reserve(count(amount)) {
                    Ok(()) => {
                        assert!(amount <= available);
                        assert_invariant(&budget, used + amount);
                    }
                    Err(error) => {
                        assert!(amount > available);
                        assert_eq!(error.requested(), count(amount));
                        assert_eq!(error.available(), count(available));
                        assert_eq!(observable(&budget), before);
                        assert_invariant(&budget, used);
                    }
                }
            }
        }
    }
}

#[test]
fn every_small_domain_release_matches_the_integer_model() {
    for limit in 0..=32 {
        for used in 0..=limit {
            for amount in 0..=33 {
                let mut budget = ByteBudget::new(count(limit));
                budget.try_reserve(count(used)).unwrap();
                let before = observable(&budget);

                match budget.release(count(amount)) {
                    Ok(()) => {
                        assert!(amount <= used);
                        assert_invariant(&budget, used - amount);
                    }
                    Err(error) => {
                        assert!(amount > used);
                        assert_eq!(error.released(), count(amount));
                        assert_eq!(error.used(), count(used));
                        assert_eq!(observable(&budget), before);
                        assert_invariant(&budget, used);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Command {
    Reserve(u64),
    Release(u64),
}

fn decode(digit: u64) -> Command {
    if digit < 5 {
        Command::Reserve(digit)
    } else {
        Command::Release(digit - 5)
    }
}

fn apply(command: Command, budget: &mut ByteBudget, model_used: &mut u64) {
    let before = observable(budget);
    let limit = budget.limit().get();

    match command {
        Command::Reserve(amount) => {
            let expected_available = limit - *model_used;
            match budget.try_reserve(count(amount)) {
                Ok(()) => {
                    assert!(amount <= expected_available);
                    *model_used += amount;
                }
                Err(error) => {
                    assert!(amount > expected_available);
                    assert_eq!(error.requested(), count(amount));
                    assert_eq!(error.available(), count(expected_available));
                    assert_eq!(observable(budget), before);
                }
            }
        }
        Command::Release(amount) => match budget.release(count(amount)) {
            Ok(()) => {
                assert!(amount <= *model_used);
                *model_used -= amount;
            }
            Err(error) => {
                assert!(amount > *model_used);
                assert_eq!(error.released(), count(amount));
                assert_eq!(error.used(), count(*model_used));
                assert_eq!(observable(budget), before);
            }
        },
    }

    assert_invariant(budget, *model_used);
}

#[test]
fn every_short_small_domain_command_sequence_matches_the_model() {
    for limit in 0..=4 {
        for depth in 0..=5_u32 {
            let sequence_count = 10_u64.pow(depth);
            for encoded in 0..sequence_count {
                let mut remaining = encoded;
                let mut budget = ByteBudget::new(count(limit));
                let mut model_used = 0;

                for _ in 0..depth {
                    let command = decode(remaining % 10);
                    remaining /= 10;
                    apply(command, &mut budget, &mut model_used);
                }
            }
        }
    }
}

#[test]
#[ignore = "qualification-tier mutation test"]
fn mutation_suite_kills_accounting_regressions()
-> Result<(), Box<dyn std::error::Error>> {
    use std::{fs, process::Command, time::SystemTime};

    const MANIFEST: &str = r#"[package]
name = "bytebudgetmutant"
version = "0.0.0"
edition = "2024"
rust-version = "1.88"

[lib]
name = "bytebudget"
path = "src/lib.rs"

[dependencies]
"#;

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let original = fs::read_to_string(root.join("src/budget.rs"))?;
    let nonce =
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_nanos();
    let mutations = [
        (
            "exact-boundary-comparison",
            "amount.get() <= self.available().get()",
            "amount.get() < self.available().get()",
        ),
        (
            "reservation-assignment",
            "self.used = ByteCount::new(self.used.get() + amount.get());",
            "self.used = self.used;",
        ),
        (
            "release-assignment",
            "self.used = ByteCount::new(self.used.get() - amount.get());",
            "self.used = self.used;",
        ),
        (
            "capacity-error-state",
            "Err(CapacityExceeded { requested: amount, available })",
            "Err(CapacityExceeded { requested: amount, available: self.used })",
        ),
        (
            "failure-state-mutation",
            "} else {\n            Err(CapacityExceeded",
            "} else {\n            self.used = self.limit;\n            Err(CapacityExceeded",
        ),
    ];

    for (index, (name, from, to)) in mutations.into_iter().enumerate() {
        assert_eq!(original.matches(from).count(), 1, "invalid mutant {name}");
        let mutant = original.replacen(from, to, 1);
        let directory = std::env::temp_dir().join(format!(
            "bytebudgetmutation{}x{nonce}x{index}",
            std::process::id()
        ));
        fs::create_dir_all(directory.join("src"))?;
        fs::create_dir_all(directory.join("tests"))?;
        fs::write(directory.join("Cargo.toml"), MANIFEST)?;
        fs::write(directory.join("src/budget.rs"), mutant)?;
        for source in ["count.rs", "lib.rs", "retained.rs"] {
            fs::copy(
                root.join("src").join(source),
                directory.join("src").join(source),
            )?;
        }
        for test in ["budget.rs", "model.rs"] {
            fs::copy(
                root.join("tests").join(test),
                directory.join("tests").join(test),
            )?;
        }

        let cargo = std::env::var_os("CARGO")
            .unwrap_or_else(|| std::ffi::OsString::from("cargo"));
        let output = Command::new(cargo)
            .current_dir(&directory)
            .args(["test", "--offline", "--test", "budget", "--test", "model"])
            .output()?;
        let reached_tests = String::from_utf8_lossy(&output.stdout)
            .contains("test result: FAILED");
        fs::remove_dir_all(&directory)?;

        assert!(!output.status.success(), "mutant survived: {name}");
        assert!(
            reached_tests,
            "mutant did not compile and reach the suite: {name}"
        );
    }

    Ok(())
}
