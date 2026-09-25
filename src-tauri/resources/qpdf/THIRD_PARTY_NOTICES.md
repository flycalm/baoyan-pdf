# Third-party notices for the bundled QPDF runtime

This application redistributes selected, unmodified runtime files from the
official **QPDF 12.4.0 MSVC 64-bit** release package.

## QPDF

QPDF is copyright (c) 2005-2021 Jay Berkenbilt and copyright (c) 2022-2026
Jay Berkenbilt and Manfred Holger. QPDF is licensed under the Apache License,
Version 2.0.

The complete license and the upstream notice are included as:

- `LICENSE.qpdf.txt`
- `NOTICE.qpdf.md`

The upstream notice also identifies the qtest, Rijndael, and sphlib-derived
code included by QPDF and states the applicable terms for those components.

## Static libraries used by the official Windows build

QPDF's official 12.4.0 Windows build uses the static dependency set published
in the same GitHub release. The exact x64 Windows dependency list is preserved
in `licenses/VCPKG_DEPENDENCIES.txt`. Runtime-relevant entries are:

- libjpeg-turbo 3.2.0 — see `licenses/LICENSE.libjpeg-turbo.txt`
- OpenSSL 3.6.3 — see `licenses/LICENSE.openssl.txt`
- zlib 1.3.2#1 — see `licenses/LICENSE.zlib.txt`

Those license files were extracted without modification from the
`x64-windows-static` portion of the release's official `vcpkg.zip` asset.

## Microsoft Visual C++ runtime

The following Microsoft Visual C++ runtime DLLs are redistributed byte for
byte as supplied in QPDF's official MSVC 64-bit package:

- `concrt140.dll`
- `msvcp140.dll`
- `msvcp140_1.dll`
- `msvcp140_2.dll`
- `msvcp140_atomic_wait.dll`
- `msvcp140_codecvt_ids.dll`
- `vcruntime140.dll`
- `vcruntime140_1.dll`

These files remain Microsoft redistributable code and are subject to the
applicable Microsoft Visual Studio licensing terms. Microsoft publishes the
supported Visual C++ redistributable information at
<https://learn.microsoft.com/cpp/windows/latest-supported-vc-redist>.

The notices above are provided for attribution and redistribution compliance;
they do not replace the referenced license texts.
