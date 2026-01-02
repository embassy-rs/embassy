
## Running the `embassy-net` examples

To run `net`, `tcp_accept`, `net_udp` and `net_dns` examples you will need a tap interface. Before running these examples, create the tap99 interface. (The number was chosen to
hopefully not collide with anything.) You only need to do this once every time you reboot your computer.

```sh
cd $EMBASSY_ROOT/examples/std/
sudo sh tap.sh
```

The example `net_ppp` requires different steps that are detailed in its section.

### `net` example

For this example, you need to have something listening in the correct port. For example `nc -lp 8000`.

Then run the example located in the `examples` folder:

```sh
cd $EMBASSY_ROOT/examples/std/
cargo run --bin net -- --tap tap99 --static-ip
```
### `tcp_accept` example

This example listen for a tcp connection.

First run the example located in the `examples` folder:

```sh
cd $EMBASSY_ROOT/examples/std/
cargo run --bin tcp_accept -- --tap tap99 --static-ip
```

Then open a connection to the port. For example `nc 192.168.69.2 9999`.

### `net_udp` example

This example listen for a udp connection.

First run the example located in the `examples` folder:

```sh
cd $EMBASSY_ROOT/examples/std/
cargo run --bin net_udp -- --tap tap99 --static-ip
```

Then open a connection to the port. For example `nc -u 192.168.69.2 9400`.

### `net_dns` example

This example queries a `DNS` for the IP address of `www.example.com`.

In order to achieve this, the `tap99` interface requires configuring tap99 as a gateway device temporarily.

For example, in Ubuntu you can do this by:

1. Identifying your default route device. In the next example `eth0`

```sh
ip r | grep "default"
default via 192.168.2.1 dev eth0 proto kernel metric 35
```

2. Enabling temporarily IP Forwarding:

```sh
sudo sysctl -w net.ipv4.ip_forward=1
```

3. Configuring NAT to mascarade traffic from `tap99` to `eth0`

```sh
sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
sudo iptables -A FORWARD -i tap99 -j ACCEPT
sudo iptables -A FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
```

4. Then you can run the example located in the `examples` folder:

```sh
cd $EMBASSY_ROOT/examples/std/
cargo run --bin net_dns -- --tap tap99 --static-ip
```

### `net_ppp` example

This example establish a Point-to-Point Protocol (PPP) connection that can be used, for example, for connecting to internet through a 4G modem via a serial channel.

The example creates a PPP bridge over a virtual serial channel between `pty1` and `pty2` for the example code and a PPP server running on the same computer. 

To run this example you will need:
- ppp (pppd server)
- socat (socket CAT)

To run the examples you may follow the next steps:

1. Save the PPP server configuration:
```sh
sudo sh -c 'echo "myuser $(hostname) mypass 192.168.7.10" >> /etc/ppp/pap-secrets'
```

2. Create a files `pty1` and `pty2` and link them 
```sh
cd $EMBASSY_ROOT/examples/std/
socat -v -x PTY,link=pty1,rawer PTY,link=pty2,rawer
```

3. open a second terminal and start the PPP server:
```sh
cd $EMBASSY_ROOT/examples/std/
sudo pppd $PWD/pty1 115200 192.168.7.1: ms-dns 8.8.4.4 ms-dns 8.8.8.8 nodetach debug local persist silent
```

4. Open a third terminal and run the example
```sh
cd $EMBASSY_ROOT/examples/std/
RUST_LOG=trace cargo run --bin net_ppp -- --device pty2
```
5. Observe the output in the second and third terminal
6. Open one last terminal to interact with `net_ppp` example through the PPP connection
```sh
# ping the net_ppp client
ping 192.168.7.10
# open an tcp connection
nc 192.168.7.10 1234
# Type anything and observe the output in the different terminals
```
