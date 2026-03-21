# Security Policy

## Dependency Security

### xz-utils (liblzma)
- **Status**: ✅ SECURE
- **Version in use**: 5.8.1+
- **Vulnerability**: CVE-2024-3156 (xz-utils backdoor in versions 5.6.0 and 5.6.1)
- **Fixed in**: 5.6.2 and later versions
- **Notes**: Our Nix environment automatically uses patched versions from nixpkgs-unstable

### OpenSSL
- **Status**: ✅ SECURE
- **Supplied by**: nixpkgs (latest patched version)
- **Environment**: Automatically set via `OPENSSL_DIR` and `OPENSSL_LIB_DIR` in flake.nix

### Rust Toolchain
- **Status**: ✅ SECURE
- **Version**: Latest stable from rust-overlay
- **Updated**: Automatically via flake inputs

### Node.js / Bun
- **Status**: ✅ SECURE
- **Version**: nodejs_22 (LTS) from nixpkgs
- **Updated**: Automatically via flake inputs

## Build Integrity

The Nix build system ensures:
1. **Reproducible Builds**: `flake.lock` pins all dependencies to exact versions
2. **Source Verification**: All packages come from official nixpkgs mirrors
3. **Isolated Environments**: No system pollution, all builds use sandboxed dependencies
4. **Automatic Updates**: `nix flake update` refreshes to latest patched versions

## Updating Security Patches

To update all dependencies to latest patched versions:

```bash
nix flake update
```

This updates `flake.lock` with the latest available versions that have passed nixpkgs security audits.

## Reporting Security Issues

If you discover a security vulnerability:
1. Do NOT open a public issue
2. Report to the project maintainers directly
3. Allow time for a patch before public disclosure

## Additional Resources

- [NixOS Security](https://nixos.org/guides/security.html)
- [CVE-2024-3156 Details](https://nvd.nist.gov/vuln/detail/CVE-2024-3156)
- [xz-utils incident timeline](https://github.com/tukaani-project/xz/commit/)
