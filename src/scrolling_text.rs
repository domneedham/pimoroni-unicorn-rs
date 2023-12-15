// // Create a new character style
// let style = MonoTextStyle::new(&FONT_5X8, Rgb888::WHITE);

// gu.clear();
// gu.draw();

// let mut x: i32 = -53;

// let message = "Pirate. Monkey. Robot. Ninja. Yolo. Wow. Cool.";

// loop {
//     delay.delay_ms(10);

//     let width = message.len() * style.font.character_size.width as usize;

//     x += 1;

//     if x > width as i32 {
//         x = -53;
//     }

//     gu.clear();
//     Text::new(message, Point::new((0 - x) as i32, 7), style)
//         .draw(&mut gu)
//         .unwrap();
//     gu.draw();

//     if gu.is_button_pressed(UnicornButtons::BrightnessUp) {
//         gu.increase_brightness(1);
//     }

//     if gu.is_button_pressed(UnicornButtons::BrightnessDown) {
//         gu.decrease_brightness(1);
//     }

//     if gu.is_button_pressed(UnicornButtons::Sleep) {
//         delay.delay_ms(2000);
//     }
// }
