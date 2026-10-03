from __future__ import annotations

import shutil
import subprocess
import unittest
from pathlib import Path


class EngineAdversarialReviewTests(unittest.TestCase):
    def test_hostile_holo_files_against_rust_engine(self) -> None:
        cargo = shutil.which("cargo")
        if cargo is None:
            self.skipTest("cargo absent: the dedicated Rust CI job remains authoritative")

        review = Path(__file__).resolve().parents[2] / "revue-langage-securite-2026-10-03"
        result = subprocess.run(
            [cargo, "test", "--manifest-path", str(review / "Cargo.toml")],
            cwd=review,
            text=True,
            capture_output=True,
            timeout=300,
            check=False,
        )
        self.assertEqual(
            result.returncode,
            0,
            "La sonde Rust des fichiers .holo hostiles a échoué.\n"
            f"stdout:\n{result.stdout}\n\nstderr:\n{result.stderr}",
        )


if __name__ == "__main__":
    unittest.main()
