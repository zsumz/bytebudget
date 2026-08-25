import { expect, smoke } from "smoque";

interface CargoMetadata {
  packages: Array<{
    name: string;
    version: string;
  }>;
}

smoke.suite(
  "publishable crate works for an external consumer",
  { tags: ["package"] },
  async (t) => {
    const root = t.repoRoot();
    const work = await t.tempDir("bytebudget-package");
    const packageTarget = work.path("package-target");

    const packageVersion = await t.step("read package identity", async () => {
      const result = await t.cmd(
        "cargo",
        [
          "+1.88.0",
          "metadata",
          "--no-deps",
          "--format-version",
          "1",
          "--locked",
          "--offline",
        ],
        { cwd: root, timeout: "30s" },
      );
      const metadata = JSON.parse(result.stdout) as CargoMetadata;
      const packageMetadata = metadata.packages.find(
        (candidate) => candidate.name === "bytebudget",
      );
      if (packageMetadata === undefined) {
        return t.fail("cargo metadata did not contain bytebudget");
      }
      return packageMetadata.version;
    });

    const packageStem = `bytebudget-${packageVersion}`;
    const packageDirectory = work.path("package-target", "package", packageStem);
    const packageManifest = work.path(
      "package-target",
      "package",
      packageStem,
      "Cargo.toml",
    );
    const archive = work.path(
      "package-target",
      "package",
      `${packageStem}.crate`,
    );
    const inspectionArchive = work.path(`${packageStem}.tar.gz`);

    await t.step("build the publishable crate", async () => {
      await t.cmd(
        "cargo",
        [
          "+1.88.0",
          "package",
          "--locked",
          "--offline",
          "--allow-dirty",
          "--target-dir",
          packageTarget,
        ],
        { cwd: root, timeout: "2m" },
      );
    });

    await t.step("inspect the archive boundary", async () => {
      await t.fs.copy(archive, inspectionArchive);
      await expect.archive(inspectionArchive).toContainEntries([
        `${packageStem}/Cargo.toml`,
        `${packageStem}/README.md`,
        `${packageStem}/bytebudget-logo.svg`,
        `${packageStem}/docs/accounting-contract.md`,
        `${packageStem}/docs/qualification.md`,
        `${packageStem}/docs/retained-charges.md`,
        `${packageStem}/LICENSES/Space-Grotesk-OFL.txt`,
        `${packageStem}/src/lib.rs`,
        `${packageStem}/tests/model.rs`,
      ]);
      await expect.archive(inspectionArchive).not.toContainEntries([
        `${packageStem}/.github/workflows/ci.yml`,
        `${packageStem}/smoke/package.smoke.ts`,
        `${packageStem}/zcheck.toml`,
        `${packageStem}/zrail.toml`,
        `${packageStem}/zrail.lock`,
      ]);
    });

    await t.step("test the extracted publishable crate", async () => {
      await t.cmd(
        "cargo",
        [
          "+1.88.0",
          "test",
          "--manifest-path",
          packageManifest,
          "--all-targets",
          "--locked",
          "--offline",
        ],
        {
          cwd: root,
          env: { CARGO_TARGET_DIR: work.path("package-test-target") },
          timeout: "2m",
        },
      );
    });

    const consumer = work.path("consumer");
    await t.step("create a fresh external consumer", async () => {
      await t.fs.mkdir(consumer);
      await t.fs.mkdir(work.path("consumer", "src"));
      await t.fs.writeText(
        work.path("consumer", "Cargo.toml"),
        `[package]
name = "bytebudgetsmokeconsumer"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
bytebudget = { path = ${JSON.stringify(packageDirectory)} }
`,
      );
      await t.fs.writeText(
        work.path("consumer", "src", "main.rs"),
        `use bytebudget::{ByteBudget, ByteCount};

fn main() {
    let mut budget = ByteBudget::new(ByteCount::new(8));
    assert_eq!(budget.try_reserve(ByteCount::new(5)), Ok(()));

    let rejection = budget
        .try_reserve(ByteCount::new(4))
        .expect_err("four bytes must not fit");
    println!(
        "used={} available={} rejected={}:{}",
        budget.used().get(),
        budget.available().get(),
        rejection.requested().get(),
        rejection.available().get(),
    );

    assert_eq!(budget.release(ByteCount::new(5)), Ok(()));
    assert_eq!(budget.used(), ByteCount::ZERO);
}
`,
      );
      await t.cmd("cargo", ["+1.88.0", "generate-lockfile", "--offline"], {
        cwd: consumer,
        timeout: "30s",
      });
    });

    await t.step("run against the packaged public API", async () => {
      const result = await t.cmd(
        "cargo",
        ["+1.88.0", "run", "--locked", "--offline"],
        {
          cwd: consumer,
          env: { CARGO_TARGET_DIR: work.path("consumer-target") },
          timeout: "2m",
        },
      );
      expect(result.stdout.trim()).toBe(
        "used=5 available=3 rejected=4:3",
      );
    });
  },
);
