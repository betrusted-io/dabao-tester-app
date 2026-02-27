#include <stdint.h>
#include <bio.h>

/*
  Dabaos are tested using a Baochip-1x. In particular, there is a test where every I/O on the PCB
  is checked for connectivity by ensuring that every wire can toggle independently. This program runs
  on a Baochip-1x integrated into the test jig itself, and reports when I/Os have been toggled.

  `cd src/c`
  `zig build -Dmodule=dabao_tester` or `python3 -m ziglang build "-Dmodule=dabao_tester"`
*/
void main(void) {
    // this pinmask includes all pins except for PC13; notably it includes the UART pins
    uint32_t pinmask = 0x1F8F783E;
    set_gpio_mask(pinmask);
    set_input_pins(pinmask);

    uint32_t status = read_gpio_pins();
    uint32_t new;
    while(1) {
        new = read_gpio_pins();
        if ((new ^ status) != 0) {
            push_fifo0(new ^ status);
            status = new ^ status;
        }
        wait_quantum();
    }
}