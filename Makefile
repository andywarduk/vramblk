all: release debug

debug:
	cargo build

release:
	cargo build --release

install: release
	sudo cp target/release/vramblk /usr/local/sbin
	sudo chmod 700 /usr/local/sbin/vramblk

systemd:
	sudo cp systemd/vramblk*.service /etc/systemd/system
	sudo chown root:root /etc/systemd/system/vramblk*
	sudo systemctl daemon-reload
	sudo systemctl enable vramblk
	sudo systemctl enable vramblk-swap
	sudo systemctl start vramblk-swap

.PHONY: systemd
