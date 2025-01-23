
## Running the `embassy-net` examples

First, create the tap99 interface. (The number was chosen to
hopefully not collide with anything.) You only need to do
this once.

```sh
sudo sh tap.sh
```

Second, have something listening there. For example `nc -lp 8000`

Then run the example located in the `examples` folder:

```sh
cd $EMBASSY_ROOT/examples/std/
sudo cargo run --bin net -- --tap tap99 --static-ip
```
