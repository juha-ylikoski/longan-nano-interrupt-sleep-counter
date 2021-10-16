#![no_std]
#![no_main]

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::jis_x0201::FONT_10X20;
use longan_nano::hal::eclic::{EclicExt, Level, Priority, TriggerType};
use longan_nano::hal::pac::{ECLIC, Interrupt, TIMER1};
use longan_nano::hal::timer::{Event, Timer};
use longan_nano::lcd::Lcd;
use panic_halt as _;

use embedded_graphics::mono_font::{
    MonoTextStyleBuilder,
};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};
use embedded_graphics::text::Text;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::{lcd, lcd_pins};
use riscv::asm;
use riscv_rt::entry;
use riscv::interrupt::{self, free};
static mut COUNTER: u32 = 0;


static mut LCD: Option<Lcd> = None;
static mut STYLE: Option<MonoTextStyle<Rgb565>> = None;
static mut TIMER1: Option<Timer<TIMER1>> = None;

fn with_lcd_and_style<T>(f: &dyn Fn(&mut Lcd, MonoTextStyle<Rgb565>) -> T) -> T {
    let lcd_ref = unsafe {
        LCD.as_mut().unwrap()
    };
    let style_ref = unsafe {
        *(STYLE.as_mut().unwrap())
    };
    f(lcd_ref, style_ref)
}


fn with_timer1<T>(f: &dyn Fn(&mut Timer<TIMER1>) -> T) -> T {
    let timer_ref = unsafe {
        TIMER1.as_mut().unwrap()
    };
    f(timer_ref)
}


#[export_name = "TIMER1"]
fn timer_1_interrupt() {
    interrupt::free(|_| {
        let val_text = heapless::String::<32>::from(unsafe {COUNTER} );
        with_lcd_and_style(&|lcd, style| Text::new(&val_text, Point::new(40, 35), style)
            .draw(lcd)
            .unwrap());

        with_timer1(&|timer1| timer1.clear_update_interrupt_flag());
        ECLIC::unpend(Interrupt::TIMER1);
        }
    );
    
}


#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();
    let mut afio = dp.AFIO.constrain(&mut rcu);

    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpiob = dp.GPIOB.split(&mut rcu);

    let lcd_pins = lcd_pins!(gpioa, gpiob);
    unsafe {
        LCD = Some(lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu));
        STYLE = Some(MonoTextStyleBuilder::new()
            .font(&FONT_10X20)
            .text_color(Rgb565::GREEN)
            .background_color(Rgb565::BLACK)
            .build());
    }

    
    let (width, height) = with_lcd_and_style(&|lcd, _| (lcd.size().width as i32, lcd.size().height as i32));
    // Clear screen
    with_lcd_and_style(&|lcd, _| Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(lcd)
        .unwrap());

    
    ECLIC::reset();
    ECLIC::setup(Interrupt::TIMER1, TriggerType::FallingEdge, Level::L0, Priority::P0);

    unsafe {
        TIMER1 = Some(Timer::timer1(dp.TIMER1, 2.hz(), &mut rcu));
    };
    

    unsafe {
        ECLIC::unmask(Interrupt::TIMER1);
    }


    with_timer1(&|timer1|timer1.listen(Event::Update));
    unsafe {interrupt::enable();}
    

    

    loop {
        free(|_|unsafe {COUNTER += 1});
        unsafe {asm::wfi()};
    }
}
