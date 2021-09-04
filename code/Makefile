build:
	RUSTFLAGS="-L /usr/lib/ -lunicorn -L /usr/local/lib/ -lkeystone -Awarnings" cargo run -- /setup/hello  /setup/rootfs/

release:
	RUSTFLAGS="-L /usr/lib/ -lunicorn -L /usr/local/lib/ -lkeystone -Awarnings" cargo run  --release -- /home/resources/boo /system/lib64/libgifimage.so /home/resources/rootfs/ /home/resources/seeds/