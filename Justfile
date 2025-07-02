import 'crates/boot/Justfile'
import 'crates/long-mode/Justfile'

WORKSPACE := source_directory()

default:
  just --list

[working-directory("iso")]
rebuild-iso:
    cp {{WORKSPACE}}/target/protected_mode/release/peluditOS {{WORKSPACE}}/iso/boot
    -(rm image.iso)
    xorriso -as mkisofs -R -r -J -b limine/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
        -apm-block-size 2048 --efi-boot limine/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        . -o image.iso
    limine bios-install image.iso

[working-directory("iso")]
qemu-run:
    qemu-system-x86_64 -boot d -cdrom image.iso -m 512

restart-os:
    just build-boot
    just rebuild-iso
    just qemu-run
    
    