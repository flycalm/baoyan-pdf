# QPDF runtime provenance and integrity

## Upstream release

- Project: QPDF
- Version: 12.4.0
- Platform: Windows x64, MSVC build
- Release: <https://github.com/qpdf/qpdf/releases/tag/v12.4.0>
- Runtime asset: `qpdf-12.4.0-msvc64.zip`
- Runtime asset size: 28,160,227 bytes
- Runtime asset SHA-256:
  `5bcb25353f7e6df92b5625dbcfe52a5c34a2a5fba2d1a8b98b8a6a0972c3ff72`
- Retrieved and verified: 2026-08-28

The archive hash matched both the `digest` field returned for the asset by the
GitHub Releases API and the `qpdf-12.4.0.sha256` file published on the same
release. An unmodified copy of that upstream checksum manifest is included as
`UPSTREAM_SHA256SUMS.txt`.

After extraction, `bin/qpdf.exe --version` returned `qpdf version 12.4.0`, and
`bin/qpdf.exe --show-crypto` returned the available providers `openssl` and
`native`.

The executable inside the upstream archive has no Authenticode signature. Its
provenance is therefore established at the official release-archive level by
the matching SHA-256 values above.

## Runtime selection

Only the QPDF command-line executable, its QPDF DLL, and every DLL shipped next
to it in the upstream `bin` directory were retained. The unrelated upstream
utilities `fix-qdf.exe` and `zlib-flate.exe`, development files, and manuals
were intentionally omitted.

The application expects the entry point at `qpdf/bin/qpdf.exe` relative to the
Tauri resource directory.

## License-source assets

The QPDF license and notice were extracted from the official release source
asset `qpdf-12.4.0.tar.gz`:

- Size: 19,675,978 bytes
- SHA-256:
  `2783a032f443cc886dad41aa6d5fae3dabf23dec00ee7ec2cfb27ef67ebcf529`

Dependency license texts and the exact dependency-version list were extracted
from the official release asset `vcpkg.zip` (`x64-windows-static` only):

- Size: 139,333,536 bytes
- SHA-256:
  `55a981f040f7e70f485a2bdd18a4a5cedeb879932c5ef5d63feec9dbaa6e1094`

Both asset hashes also match `UPSTREAM_SHA256SUMS.txt` and the corresponding
GitHub Releases API asset digests. No executable or library from `vcpkg.zip`
was bundled; only its dependency list and license texts were retained.
