#!/bin/python3

import tomllib
import subprocess

def version():
    with open("Cargo.toml", "rb") as f:
        data = tomllib.load(f)
        if pkg := data.get("package"):
            return pkg.get("version")
    return None

if __name__ == "__main__":
    choice = input(f"Do you want to publish version {version()} to crates.io [y/N] ").strip().lower() == 'y'
    if choice:
        subprocess.run(["cargo", "publish"])
