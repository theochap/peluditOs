import 'crates/boot/Justfile'
import 'crates/long-mode/Justfile'

WORKSPACE := source_directory()

default:
  just --list

[working-directory("iso")]
rebuild-iso:
    cp {{WORKSPACE}}/target/x86_64-unknown-none/release/peluditOS_x86_64 {{WORKSPACE}}/iso/boot/peluditOS
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
    just build-long
    just rebuild-iso
    just qemu-run
    
    