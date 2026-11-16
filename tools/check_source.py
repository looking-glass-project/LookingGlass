"""Audit the Git index before a source commit. Read-only; never uploads."""
from pathlib import Path, PurePosixPath
import argparse
import hashlib
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
ROOT_FILES = {
    ".gitignore", ".gitattributes", "Cargo.toml", "Cargo.lock", "README.md",
    "LEGAL.md", "LICENSE-MIT", "Setup.cmd", "CONTRIBUTING.md", "SECURITY.md",
    "THIRD_PARTY_NOTICES.txt", "build.rs",
}
SUFFIXES = {"src": {".rs", ".json"}, "docs": {".md"}, "tools": {".py", ".ps1", ".java", ".cmd", ".md"}}
# Authored public website files only; media retains its exact hash boundary.
SITE_FILES = {
    "docs/site/index.html", "docs/site/styles.css", "docs/site/favicon.svg",
    "docs/site/sitemap.xml",
}
# User-requested README footage. Exact files/content only, never a general
# allowance for original game assets or raw research captures.
GAMEPLAY_MEDIA = {
    "docs/media/gameplay/story-preview.gif": "697ceef7f8108cbfd3521c7892b9db5a9a6da8a79eb169aefac346547440db5d",
    "docs/media/gameplay/story-poster.jpg": "51c96dff0690f5908a786cfe2b4eed7453db79fea8c89e9d364a64f47037eeed",
    "docs/media/gameplay/dry-landing.mp4": "e04e476846a5e12b947ffba3616eefd5a8759d82cbd927182c9d0dc3d2da08fe",
    "docs/media/gameplay/dry-landing.gif": "b1fca02df8d1714c007d00932f644c778fafd69d9883475129135cc7049bd6c4",
    "docs/media/gameplay/pool-of-tears.mp4": "5840a570947d79472e9b4087344f06a3d35110fdf1d64f9c29b9826f49f4dc2b",
    "docs/media/gameplay/pool-of-tears.gif": "707bae12ba4cdbbff78d0b4588c717d7ee988b7fd70c2d0c395aa0b22f1cc839",
    "docs/media/gameplay/pale-realm.mp4": "864262da7717b8e193be1f6f6f4f994de90b1781b8c804da62869f678b9102b0",
    "docs/media/gameplay/pale-realm.gif": "56e0ef4b94b96493212ac2fe39369b458960d6c2212c2bf512246e5cad74d5fe",
}
SECRET = re.compile(
    rb"(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}|"
    rb"AKIA[A-Z0-9]{16}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----)"
)


# PS5 port (ps5/README.md).
PS5_FILES = {"ps5/.gitignore", "ps5/sce_sys/icon0.png", "vendor/README.md"}
# Binary files of the port, by exact content: the title's icon (PS5_VulkanTemplate's placeholder)
# and the resources macroquad embeds with include_bytes!.
PINNED_BINARIES = {
    "ps5/sce_sys/icon0.png": "902be3702d17039ffc6320cf53f94d54abd393d26c2e19fa91a5548a613b5417",
    "vendor/macroquad/src/ProggyClean.ttf": "527d2a443ce051f93f7e77b855609722b8cb220a9f104b4aa037be5c90b71324",
    "vendor/macroquad/src/font.png": "634ed3801b905c8fa55b3abd3ff3c319d629712599928b62128061c8823cce78",
    "vendor/macroquad/src/ui/combobox.img": "881379bc94e541ee0e1f912ec609edacdb99571f2957036cd330f6748c027d50",
}
VENDORED = {"miniquad", "macroquad", "cpal", "ps5-std-rt"}
VENDOR_LICENCES = {"LICENSE", "LICENSE-MIT", "LICENSE-APACHE"}
VENDOR_RESOURCES = set(PINNED_BINARIES)


def allowed(path):
    p = PurePosixPath(path)
    if p.is_absolute() or ".." in p.parts or "\\" in path:
        return False
    if path in GAMEPLAY_MEDIA or path in SITE_FILES or path in PS5_FILES:
        return True
    if len(p.parts) == 1:
        return path in ROOT_FILES or (path.startswith("Launch") and p.suffix == ".cmd")
    if p.parts[0] in SUFFIXES:
        return p.suffix in SUFFIXES[p.parts[0]] and "__pycache__" not in p.parts
    if p.parts[0] == "ps5" and len(p.parts) > 1:
        # PS5 port: scripts, docs, the title's metadata/icon and the backend test program.
        return path in PS5_FILES or (p.suffix in {".sh", ".md", ".json", ".toml", ".rs"} and "__pycache__" not in p.parts)
    if p.parts[0] == "vendor" and len(p.parts) > 2 and p.parts[1] in VENDORED:
        # Patched third-party crates (vendor/README.md): source, manifests, licences, and
        # the few binary resources the crates themselves include at compile time.
        return (p.suffix in {".rs", ".toml", ".md", ".c", ".h", ".js", ".java", ".lock"}
                or p.name in VENDOR_LICENCES or path in VENDOR_RESOURCES)
    return p.parts[:2] in {(".github", "workflows"), (".github", "ISSUE_TEMPLATE")} and p.suffix in {".yml", ".yaml", ".md"}


def size_limit(path):
    return (8 if path in GAMEPLAY_MEDIA and path.endswith(".mp4") else 2) * 1024 * 1024


def audit_data(path, data):
    errors = []
    if not allowed(path):
        errors.append(f"Not an approved source path: {path}")
    if len(data) > size_limit(path):
        errors.append(f"Unexpectedly large source file: {path}")
    if path in GAMEPLAY_MEDIA:
        if hashlib.sha256(data).hexdigest() != GAMEPLAY_MEDIA[path]:
            errors.append(f"Unreviewed gameplay media content: {path}")
    elif path in PINNED_BINARIES:
        if hashlib.sha256(data).hexdigest() != PINNED_BINARIES[path]:
            errors.append(f"Unreviewed binary content: {path}")
    elif b"\0" in data:
        errors.append(f"Binary content in source file: {path}")
    if SECRET.search(data):
        errors.append(f"Possible credential/private key in: {path}")
    if path.endswith(".json"):
        try:
            json.loads(data)
        except (ValueError, UnicodeDecodeError):
            errors.append(f"Invalid JSON source data: {path}")
    return errors


def working_files(root=ROOT):
    """Only Git-visible files; include new source, omit deleted and ignored files."""
    names = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=root
    ).decode("utf-8").split("\0")
    result = []
    for name in sorted(set(names) - {""}):
        p = root / name
        if not p.exists() and not p.is_symlink():
            continue
        if p.is_symlink() or not p.is_file() or not p.resolve().is_relative_to(root.resolve()):
            raise ValueError(f"Not an ordinary source file: {name}")
        result.append(p)
    return result


def index_blobs(entries, root=ROOT):
    """Read staged blobs through two Git processes, retaining per-file limits."""
    if not entries:
        return
    sizes = subprocess.check_output(
        ["git", "cat-file", "--batch-check=%(objectsize)"], cwd=root,
        input="".join(f"{blob}\n" for _, blob in entries).encode("ascii"),
    ).splitlines()
    if len(sizes) != len(entries):
        raise RuntimeError("Incomplete Git object size response")
    with subprocess.Popen(["git", "cat-file", "--batch"], cwd=root,
                          stdin=subprocess.PIPE, stdout=subprocess.PIPE) as reader:
        for (path, blob), raw_size in zip(entries, sizes):
            size = int(raw_size)
            if size > size_limit(path):
                yield path, size, None
                continue
            reader.stdin.write(f"{blob}\n".encode("ascii"))
            reader.stdin.flush()
            header = reader.stdout.readline().split()
            if header != [blob.encode("ascii"), b"blob", str(size).encode("ascii")]:
                raise RuntimeError(f"Invalid Git blob response for {path}")
            data = reader.stdout.read(size)
            if len(data) != size or reader.stdout.read(1) != b"\n":
                raise RuntimeError(f"Incomplete Git blob response for {path}")
            yield path, size, data
        reader.stdin.close()
        if reader.wait() != 0:
            raise RuntimeError("Git blob reader failed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worktree", action="store_true", help="audit current tracked and new files without staging")
    args = parser.parse_args()
    if args.worktree:
        files = working_files()
        errors = []
        for p in files:
            errors.extend(audit_data(p.relative_to(ROOT).as_posix(), p.read_bytes()))
        if errors:
            print("Source worktree audit failed:\n" + "\n".join(errors))
            return 1
        print(f"Source worktree audit passed: {len(files)} ordinary allowlisted files.")
        print("This does not establish full-history or original-text provenance review.")
        return 0
    index = subprocess.check_output(["git", "ls-files", "--stage", "-z"], cwd=ROOT)
    errors = []
    entries = []
    count = total = 0
    for record in index.split(b"\0"):
        if not record:
            continue
        metadata, raw_path = record.split(b"\t", 1)
        mode, blob, stage = metadata.decode("ascii").split()
        path = raw_path.decode("utf-8")
        count += 1
        if mode not in {"100644", "100755"} or stage != "0" or not allowed(path):
            errors.append(f"Not an approved ordinary source file: {path}")
            continue
        entries.append((path, blob))
    for path, size, data in index_blobs(entries):
        if data is None:
            errors.append(f"Unexpectedly large source file: {path}")
            continue
        total += len(data)
        errors.extend(audit_data(path, data))
    if errors:
        print("Source audit failed:\n" + "\n".join(errors))
        return 1
    print(f"Source index audit passed: {count} files, {total:,} bytes; no disallowed paths/binaries or recognised credential patterns.")
    print("This checks the index, not all history, provenance or every possible secret.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
