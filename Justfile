build:
    cross build --target i686-pc-windows-gnu
    cp /mnt/storage/cargo-target/i686-pc-windows-gnu/debug/skrillax-client-patcher.exe /home/tim/Games/silkroad-online/drive_c/Program\ Files\ \(x86\)/Silkroad/

fmt:
    cargo fmt
