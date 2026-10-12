#!/usr/bin/env python3
import os
import shutil
import subprocess
import sys
from pathlib import Path

LIMINE_FILES = [
    "limine-bios-cd.bin",
    "limine-uefi-cd.bin",
    "limine-bios.sys",
    "BOOTX64.EFI",
]


def find_project_root() -> Path:
    here = Path(__file__).resolve().parent
    for parent in [here, *here.parents]:
        if (parent / "Cargo.toml").exists():
            return parent
    sys.exit("[build] error: Cargo.toml not found above " + str(here))


def find_exe(name: str, hint: str) -> str:
    path = shutil.which(name)
    if path is None:
        sys.exit(f"[build] error: '{name}' not found in PATH.\n[build] {hint}")
    return path


def run(cmd, cwd: Path) -> None:
    print("[build] $", " ".join(str(c) for c in cmd), flush=True)
    subprocess.run([str(c) for c in cmd], cwd=str(cwd), check=True)


def main() -> None:
    if len(sys.argv) < 2:
        sys.exit("usage: build_and_run.py <kernel-elf-path>")

    project_root = find_project_root()
    kernel_src = Path(sys.argv[1]).resolve()
    build_dir = project_root / "build"
    iso_root = build_dir / "iso_root"
    iso_image = build_dir / "aloha.iso"
    limine_src = project_root / "tools" / "limine"
    conf_src = project_root / "limine.conf"

    if not kernel_src.is_file():
        sys.exit(f"[build] error: kernel ELF not found: {kernel_src}")

    missing = [f for f in LIMINE_FILES if not (limine_src / f).exists()]
    if missing:
        sys.exit("[build] error: tools/limine incomplete, missing:\n  " + "\n  ".join(missing))
    if not conf_src.exists():
        sys.exit("[build] error: limine.conf not found in project root")

    if iso_root.exists():
        shutil.rmtree(iso_root)
    (iso_root / "boot" / "limine").mkdir(parents=True)
    (iso_root / "EFI" / "BOOT").mkdir(parents=True)

    for name in LIMINE_FILES:
        if name == "BOOTX64.EFI":
            dst = iso_root / "EFI" / "BOOT" / name
        else:
            dst = iso_root / "boot" / "limine" / name
        shutil.copy2(limine_src / name, dst)
    shutil.copy2(conf_src, iso_root / "boot" / "limine" / "limine.conf")

    kernel_dst = iso_root / "boot" / "kernel.elf"
    print(f"[build] copy {kernel_src.name} -> {kernel_dst}", flush=True)
    shutil.copy2(kernel_src, kernel_dst)

    if iso_image.exists():
        try:
            iso_image.unlink()
        except PermissionError:
            sys.exit(f"[build] error: cannot delete {iso_image} - is QEMU still running?")

    xorriso = find_exe("xorriso", "install: Windows: scoop/choco install xorriso | Debian/Ubuntu: apt install xorriso | Arch: pacman -S xorriso | macOS: brew install xorriso")
    xorriso_cmd = [
        xorriso, "-as", "mkisofs",
        "-r",
        "-b", "boot/limine/limine-bios-cd.bin",
        "-no-emul-boot", "-boot-load-size", "4", "-boot-info-table",
        "--efi-boot", "boot/limine/limine-uefi-cd.bin",
        "-efi-boot-part", "--efi-boot-image", "--protective-msdos-label",
        "iso_root",
        "-o", str(iso_image),
    ]
    if os.name == "nt" and xorriso.lower().endswith((".cmd", ".bat")):
        xorriso_cmd = ["cmd", "/c"] + xorriso_cmd

    try:
        run(xorriso_cmd, build_dir)
    except subprocess.CalledProcessError:
        print(f"[build] FAILED - iso_root kept for diagnostics: {iso_root}", flush=True)
        sys.exit(1)

    shutil.rmtree(iso_root)
    print(f"[build] ok - {iso_image}", flush=True)

    ovmf_code = project_root / "tools" / "ovmf" / "OVMF_CODE.fd"
    ovmf_vars = project_root / "tools" / "ovmf" / "OVMF_VARS.fd"
    if not ovmf_code.exists() or not ovmf_vars.exists():
        sys.exit("[build] error: OVMF not found in tools/ovmf/")

    qemu = find_exe("qemu-system-x86_64", "install: Windows: choco install qemu | Debian/Ubuntu: apt install qemu-system-x86 | Arch: pacman -S qemu | macOS: brew install qemu")
    run(
        [qemu,
         "-drive", f"file={ovmf_code},if=pflash,format=raw,readonly=on",
         "-drive", f"file={ovmf_vars},if=pflash,format=raw",
         "-cdrom", str(iso_image),
         "-serial", "stdio",
         "-m", "256M",
         "-no-reboot"],
        project_root,
    )


if __name__ == "__main__":
    main()