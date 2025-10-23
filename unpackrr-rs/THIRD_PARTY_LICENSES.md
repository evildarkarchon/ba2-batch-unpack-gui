# Third-Party Licenses

Unpackrr includes or depends on the following third-party software components. We are grateful to the developers and maintainers of these projects.

---

## Bundled Software

### BSArch.exe

**Purpose**: BA2 archive extraction and listing
**Source**: [TES5Edit Project](https://github.com/TES5Edit/TES5Edit)
**License**: Mozilla Public License 2.0 (MPL-2.0)
**Copyright**: TES5Edit Team

BSArch.exe is bundled with Unpackrr to provide reliable BA2 extraction functionality. It is a mature, battle-tested tool that handles the complex BA2 format with all its edge cases.

**MPL-2.0 License Summary**:
- ✅ Commercial use permitted
- ✅ Modification permitted
- ✅ Distribution permitted
- ✅ Patent use permitted
- ✅ Private use permitted
- ⚠️  Source code must be made available when distributed
- ⚠️  Modified files must carry notices of changes
- ⚠️  License and copyright notice must be included

**Full License Text**: https://www.mozilla.org/en-US/MPL/2.0/

**Attribution**:
```
BSArch - Bethesda Archive Tool
Copyright (c) TES5Edit Team
Licensed under MPL-2.0
https://github.com/TES5Edit/TES5Edit
```

---

## Rust Dependencies

The following Rust crates are used in this project. All are statically linked into the final binary.

### GUI Framework

#### Slint
- **Version**: 1.14.0
- **License**: GPL-3.0 or Commercial
- **Source**: https://github.com/slint-ui/slint
- **Purpose**: Declarative UI framework with Fluent Design support

---

### Async Runtime

#### Tokio
- **Version**: 1.41+
- **License**: MIT
- **Source**: https://github.com/tokio-rs/tokio
- **Purpose**: Asynchronous runtime for concurrent operations

#### async-compat
- **Version**: 0.2
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/smol-rs/async-compat
- **Purpose**: Bridge between Tokio and Slint event loops

---

### Error Handling

#### anyhow
- **Version**: 1.0
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/dtolnay/anyhow
- **Purpose**: Flexible error handling

#### thiserror
- **Version**: 2.0
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/dtolnay/thiserror
- **Purpose**: Derive macros for error types

---

### Serialization

#### serde
- **Version**: 1.0
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/serde-rs/serde
- **Purpose**: Serialization framework

#### serde_json
- **Version**: 1.0
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/serde-rs/json
- **Purpose**: JSON serialization

#### toml
- **Version**: 0.9
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/toml-rs/toml
- **Purpose**: TOML configuration format support

---

### Utilities

#### regex
- **Version**: 1.11
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/rust-lang/regex
- **Purpose**: Regular expression support for file filtering

#### rayon
- **Version**: 1.10
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/rayon-rs/rayon
- **Purpose**: Parallel processing for BA2 scanning

#### directories
- **Version**: 6.0
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/dirs-dev/directories-rs
- **Purpose**: Platform-specific directory paths

#### dunce
- **Version**: 1.0
- **License**: CC0-1.0 OR MIT OR Apache-2.0
- **Source**: https://github.com/kornelski/dunce
- **Purpose**: Windows UNC path handling

#### humansize
- **Version**: 2.1
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/LeopoldArkham/humansize
- **Purpose**: Human-readable file size formatting

#### memmap2
- **Version**: 0.9
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/RazrFalcon/memmap2-rs
- **Purpose**: Memory-mapped file I/O for large BA2 files

---

### Logging

#### tracing
- **Version**: 0.1
- **License**: MIT
- **Source**: https://github.com/tokio-rs/tracing
- **Purpose**: Structured logging and diagnostics

#### tracing-subscriber
- **Version**: 0.3
- **License**: MIT
- **Source**: https://github.com/tokio-rs/tracing
- **Purpose**: Log collection and formatting

#### tracing-appender
- **Version**: 0.2
- **License**: MIT
- **Source**: https://github.com/tokio-rs/tracing
- **Purpose**: Daily rotating log files

---

### File Dialogs

#### rfd
- **Version**: 0.15
- **License**: MIT
- **Source**: https://github.com/PolyMeilex/rfd
- **Purpose**: Native file/folder picker dialogs

---

### Networking

#### reqwest
- **Version**: 0.12
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/seanmonstar/reqwest
- **Purpose**: HTTP client for update checking

#### semver
- **Version**: 1.0
- **License**: MIT OR Apache-2.0
- **Source**: https://github.com/dtolnay/semver
- **Purpose**: Semantic version comparison

---

### Platform Integration

#### open
- **Version**: 5.0
- **License**: MIT
- **Source**: https://github.com/Byron/open-rs
- **Purpose**: Cross-platform "open file/folder" functionality

#### winreg (Windows only)
- **Version**: 0.52
- **License**: MIT
- **Source**: https://github.com/gentoo90/winreg-rs
- **Purpose**: Windows registry access for default BA2 handler detection

---

## License Compatibility

All dependencies are compatible with Unpackrr's GPL-3.0 license:

- **MIT License**: Compatible with GPL-3.0 (permissive)
- **Apache-2.0 License**: Compatible with GPL-3.0 (permissive)
- **MPL-2.0 License**: Compatible with GPL-3.0 (weak copyleft)
- **GPL-3.0 License**: Same license (Slint)

---

## Full License Texts

### MIT License

```
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Apache License 2.0

Full text available at: https://www.apache.org/licenses/LICENSE-2.0

Key points:
- Commercial use, modification, distribution, patent use, and private use permitted
- Must include license and copyright notice
- Must state changes made to the software
- Provides an express grant of patent rights

### Mozilla Public License 2.0 (MPL-2.0)

Full text available at: https://www.mozilla.org/en-US/MPL/2.0/

Key points:
- File-level copyleft (only modified files must be open-sourced)
- Commercial use, modification, distribution permitted
- Patent grant included
- Compatible with GPL-3.0 when combined with other GPL-3.0 code

---

## Acknowledgments

We are deeply grateful to all the developers and maintainers of these projects. Open source software like this makes projects like Unpackrr possible.

Special thanks to:
- **TES5Edit Team** for BSArch.exe
- **Slint Team** for the GUI framework
- **Tokio Team** for the async runtime
- All contributors to the Rust crate ecosystem

---

## Questions?

If you have questions about licensing or third-party components, please:
- Open an issue: https://github.com/evildarkarchon/ba2-batch-unpack-gui/issues
- Contact the maintainer: evildarkarchon

---

**Last Updated**: 2025-10-22
**Unpackrr Version**: 0.1.0
