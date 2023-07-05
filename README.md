# sim800-rs 
<img src="./assets/AM-016.jpg" width="300">


**SIM800 Family (quad-band GSM/GPRS module(s)) embedded-hal and ```no_std``` driver written in Rust**

The idea is to develop a driver for the SIM800 family in Rust that can be used in both ```std``` and ```no_std``` environments.  For the development a module with UART-TO-USB is used and the access is abstracted by using [linux-embedded-hal](https://github.com/rust-embedded/linux-embedded-hal).




