# ArchLume development tasks.
# Run `just --list` to see available recipes.
set shell := ["bash", "-uc"]

default:
	@just --list

# Build the whole Cargo workspace
build:
	cargo build --workspace

# Build in release mode
build-release:
	cargo build --workspace --release

# Run all tests
test:
	cargo test --workspace

# Format all code
fmt:
	cargo fmt --all

# Check formatting without writing changes
fmt-check:
	cargo fmt --all -- --check

# Lint with warnings denied
lint:
	cargo clippy --workspace --all-targets -- -D warnings

# Full local gate (format + lint + test)
check: fmt-check lint test
	@echo "check: OK"

# Run the compositor nested in the current Wayland session (dev only)
run-nested:
	cargo run -p archlume-compositor

# Nested smoke test: start the compositor and verify a client maps a window
smoke:
	bash scripts/smoke-nested.sh

# Runtime test of the control socket (drives workspace state without input events)
control-test:
	bash scripts/control-test.sh

# Boot the newest ISO in QEMU (UEFI) and verify the live environment via OCR
boot-test:
	bash scripts/boot-iso-qemu.sh

# Build the archiso profile into iso/out/
iso:
	@command -v mkarchiso >/dev/null || { echo "mkarchiso not found (install the 'archiso' package)"; exit 1; }
	@test -d iso/profile || { echo "iso/profile not present yet (Milestone 2)"; exit 1; }
	sudo mkarchiso -v -w iso/work -o iso/out iso/profile

# Boot the most recent built ISO in QEMU with a disposable virtual disk.
# The host's real disks are never passed through.
qemu:
	@command -v qemu-system-x86_64 >/dev/null || { echo "qemu not found (install the 'qemu-desktop' package)"; exit 1; }
	iso_file=$$(ls -t iso/out/*.iso 2>/dev/null | head -1); \
	test -n "$$iso_file" || { echo "no ISO in iso/out/; run 'just iso' first"; exit 1; }; \
	test -f /usr/share/edk2/x64/OVMF_CODE.4m.fd || { echo "OVMF firmware not found (install 'edk2-ovmf')"; exit 1; }; \
	cp -f /usr/share/edk2/x64/OVMF_VARS.4m.fd /tmp/archlume-OVMF_VARS.fd; \
	qemu-system-x86_64 \
		-enable-kvm -m 4096 -cpu host -smp 2 \
		-drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/x64/OVMF_CODE.4m.fd \
		-drive if=pflash,format=raw,file=/tmp/archlume-OVMF_VARS.fd \
		-drive file="$$iso_file",media=cdrom,readonly=on \
		-drive file=/tmp/archlume-test-disk.qcow2,if=virtio,format=qcow2 \
		-device virtio-vga-gl -display gtk,gl=on \
		-netdev user,id=n0 -device virtio-net-pci,netdev=n0 \
		-boot menu=on
