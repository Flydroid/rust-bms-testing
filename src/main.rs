#![deny(warnings)]
#![no_main]
#![no_std]


mod sys; // imports the sys module
mod panic; //imports the panic handler

#[allow(unused)]
use cortex_m_rt::entry;
//use nb::block
/* Select which parts to use from the stm327xx_hal */
use stm32h7xx_hal::{pac, prelude::*, rcc,gpio::{Alternate, Pin}, time::Hertz,spi}; 
/*Setup RTT Logging */
pub use rtt_target::{rprintln as log, rtt_init_print as log_init};

use ltc681x::ltc6812::{CellSelection, LTC6812};
use ltc681x::monitor::{ADCMode, LTC681X, LTC681XClient, PollClient};
/* Define processor frequency */
pub const CORE_FREQUENCY: Hertz = Hertz::from_raw(480_000_000);


#[entry]
fn main() -> ! {
    log_init!(); // Init rtt logging

    sys::Clk::new().reset().enable_ext_clock();

    
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    log!("Setup Power...");
    // Constrain and Freeze power
    let pwrcfg: stm32h7xx_hal::pwr::PowerConfiguration = dp.PWR.constrain().vos0(&dp.SYSCFG).freeze();
    log!("Setup RCC...");
    // Constrain and Freeze clock
    let ccdr = dp
        .RCC
        .constrain()
        .use_hse(25.MHz())
        .bypass_hse()
        .sys_ck(CORE_FREQUENCY)
        .hclk(240.MHz())
        .pll1_strategy(rcc::PllConfigStrategy::Iterative)
        .freeze(pwrcfg, &dp.SYSCFG);

    /*Get GPIO K peripheral */
    let gpio_k = dp.GPIOK.split(ccdr.peripheral.GPIOK);
    /*Get GPIO J peripheral */
    let gpio_j = dp.GPIOJ.split(ccdr.peripheral.GPIOJ);
    /*Get GPIO J peripheral */
    let gpio_h = dp.GPIOH.split(ccdr.peripheral.GPIOH);

    let mosi: Pin<'J', 10, Alternate<5>> = gpio_j.pj10.into_alternate();
    let miso: Pin<'J', 11, Alternate<5>> = gpio_j.pj11.into_alternate();
    let sck:  Pin<'H', 6, Alternate<5>>  = gpio_h.ph6.into_alternate();

    let mut cs_pin: stm32h7xx_hal::gpio::Pin<'K', 1, stm32h7xx_hal::gpio::Output> = gpio_k.pk1.into_push_pull_output();
    
    // Initialise the SPI peripheral.
    let mut spi_bus: spi::Spi<pac::SPI5, spi::Enabled> = dp.SPI5.spi(
        (sck,miso,mosi),
        spi::MODE_0,
        1u32.MHz(),
        ccdr.peripheral.SPI5,
        &ccdr.clocks,

    );

    // LTC6812 device
    let mut client: LTC681X<_, _, _, LTC6812, 1> = LTC681X::ltc6812(spi_bus, cs_pin)
        .enable_sdo_polling();

    // Starts conversion for cell group 1
    client.start_conv_cells(ADCMode::Normal, CellSelection::Group1, true);

    // Configure PK5 as output.
    let mut led = gpio_k.pk5.into_push_pull_output();

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);






    log!("Setup Done");

    // Echo what is received on the SPI
    let mut received = 0;

    loop {
        // Poll ADC status
        while !client.adc_ready().unwrap() {
            // Conversion is not done yet
            log!("Waiting for LTC6812");
            delay.delay_ms(10_u16);
        }

        // Returns the value of cell group A. In case of LTC613: cell 1, 7 and 13
        let voltages = client.read_voltages(CellSelection::Group1).unwrap();


        led.set_high();
        delay.delay_ms(10_u16);
        log!("Blink!");
        led.set_low();
        delay.delay_ms(10_u16);
    }

}