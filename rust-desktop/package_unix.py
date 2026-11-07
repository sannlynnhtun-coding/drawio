"""Build native macOS DMG or Ubuntu 24.04 x64 DEB packages using OS tools."""
import argparse
import hashlib
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tempfile
import tomllib


def run(*args, **kwargs):
    return subprocess.check_output(args, text=True, **kwargs).strip()


def macos(stage, binary, output, version, arch, repo):
    app = stage / "Drawio.app"
    contents = app / "Contents"
    (contents / "MacOS").mkdir(parents=True)
    resources = contents / "Resources"
    resources.mkdir()
    shutil.copy2(binary, contents / "MacOS/Drawio")
    shutil.copy2(repo / "LICENSE", resources / "LICENSE")
    plist = {
        "CFBundleIdentifier": "com.sannlynnhtun.drawio",
        "CFBundleName": "Drawio",
        "CFBundleDisplayName": "Drawio",
        "CFBundleExecutable": "Drawio",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": version,
        "CFBundleVersion": version,
        "CFBundleIconFile": "Drawio.icns",
        "LSMinimumSystemVersion": "14.0",
        "NSHighResolutionCapable": True,
        "CFBundleDocumentTypes": [{
            "CFBundleTypeName": "Drawio Diagram",
            "CFBundleTypeRole": "Editor",
            "LSHandlerRank": "Alternate",
            "LSItemContentTypes": ["com.sannlynnhtun.drawio.diagram"],
        }],
        "UTExportedTypeDeclarations": [{
            "UTTypeIdentifier": "com.sannlynnhtun.drawio.diagram",
            "UTTypeDescription": "Drawio Diagram",
            "UTTypeConformsTo": ["public.xml"],
            "UTTypeTagSpecification": {"public.filename-extension": ["drawio"]},
        }],
    }
    (contents / "Info.plist").write_bytes(plistlib.dumps(plist))
    iconset = stage.parent / "Drawio.iconset"
    iconset.mkdir()
    source = repo / "src/main/webapp/images/drawlogo256.png"
    for size in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            name = f"icon_{size}x{size}{'@2x' if scale == 2 else ''}.png"
            run("sips", "-z", str(size * scale), str(size * scale), str(source), "--out", str(iconset / name))
    run("iconutil", "-c", "icns", str(iconset), "-o", str(resources / "Drawio.icns"))
    expected = "arm64" if arch == "arm64" else "x86_64"
    if run("lipo", "-archs", str(contents / "MacOS/Drawio")) != expected:
        raise RuntimeError("Mac executable architecture does not match its package name")
    run("codesign", "--force", "--deep", "--sign", "-", str(app))
    run("codesign", "--verify", "--deep", "--strict", str(app))
    (stage / "Applications").symlink_to("/Applications", target_is_directory=True)
    package = output / f"Drawio-macOS-{arch}.dmg"
    run("hdiutil", "create", "-volname", "Drawio", "-srcfolder", str(stage), "-format", "UDZO", "-ov", str(package))
    run("hdiutil", "verify", str(package))
    return package


def linux(stage, binary, output, version, arch, repo):
    if arch != "x64":
        raise RuntimeError("The Linux release currently targets x64")
    files = {
        "usr/bin/Drawio": binary,
        "usr/share/doc/drawio/copyright": repo / "LICENSE",
        "usr/share/icons/hicolor/256x256/apps/drawio.png": repo / "src/main/webapp/images/drawlogo256.png",
    }
    for name, source in files.items():
        dest = stage / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, dest)
    desktop = stage / "usr/share/applications/drawio.desktop"
    desktop.parent.mkdir(parents=True)
    desktop.write_text("[Desktop Entry]\nType=Application\nName=Drawio\nComment=Offline diagram editor\nExec=Drawio %f\nIcon=drawio\nStartupWMClass=drawio\nTerminal=false\nCategories=Graphics;\nMimeType=application/x-drawio;\n", encoding="utf-8")
    mime = stage / "usr/share/mime/packages/drawio.xml"
    mime.parent.mkdir(parents=True)
    mime.write_text('<?xml version="1.0"?><mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info"><mime-type type="application/x-drawio"><comment>Drawio Diagram</comment><glob pattern="*.drawio"/></mime-type></mime-info>', encoding="utf-8")
    run("desktop-file-validate", str(desktop))
    debian = stage / "debian"
    debian.mkdir()
    (debian / "control").write_text("Source: drawio\nSection: graphics\nPriority: optional\nMaintainer: Drawio contributors\n\nPackage: drawio\nArchitecture: amd64\nDescription: Offline diagram editor\n")
    shlibs = run("dpkg-shlibdeps", "-O", "-eusr/bin/Drawio", cwd=stage)
    depends = next(line.split("=", 1)[1] for line in shlibs.splitlines() if line.startswith("shlibs:Depends="))
    control = stage / "DEBIAN"
    control.mkdir()
    (control / "control").write_text(f"Package: drawio\nVersion: {version}\nSection: graphics\nPriority: optional\nArchitecture: amd64\nMaintainer: Drawio contributors\nDepends: {depends}, libvulkan1, shared-mime-info\nRecommends: mesa-vulkan-drivers\nDescription: Offline diagram editor\n Rust and GPUI Kit desktop app with the local draw.io web editor.\n")
    shutil.rmtree(debian)
    package = output / "Drawio-Linux-x64.deb"
    run("dpkg-deb", "--root-owner-group", "--build", str(stage), str(package))
    if run("dpkg-deb", "--field", str(package), "Version") != version:
        raise RuntimeError("DEB version mismatch")
    if run("dpkg-deb", "--field", str(package), "Architecture") != "amd64":
        raise RuntimeError("DEB architecture mismatch")
    return package


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--arch", required=True, choices=("x64", "arm64"))
    parser.add_argument("--output", type=Path, default=Path("rust-desktop/target/dist"))
    args = parser.parse_args()
    if sys.platform not in ("darwin", "linux"):
        parser.error("Run this packager on macOS or Linux")
    repo = Path(__file__).resolve().parent.parent
    version = tomllib.loads((repo / "rust-desktop/Cargo.toml").read_text())["package"]["version"]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    binary = args.binary.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="drawio-package-", dir=output.parent) as temp:
        stage = Path(temp) / "stage"
        stage.mkdir()
        package = (macos if sys.platform == "darwin" else linux)(stage, binary, output, version, args.arch, repo)
    digest = hashlib.file_digest(package.open("rb"), "sha256").hexdigest()
    package.with_suffix(package.suffix + ".sha256").write_text(f"{digest}  {package.name}\n", encoding="ascii")
    print(package)


if __name__ == "__main__":
    main()
