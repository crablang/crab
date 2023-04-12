// crablangfmt-indent_style: Block
// crablangfmt-max_width: 80
// crablangfmt-tab_spaces: 2

// #1427
fn main() {
  exceptaions::config(move || {
    (
      NmiConfig {},
      HardFaultConfig {},
      SysTickConfig { gpio_sbsrr },
    )
  });
}
