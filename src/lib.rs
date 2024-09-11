use html2text::from_read;
use nvim_oxi::api::{self, opts::*, types::*, Buffer, Window};
use nvim_oxi::{print, Dictionary, Function};
use std::cell::RefCell;
use std::rc::Rc;

#[nvim_oxi::plugin]
fn devdocs() -> nvim_oxi::Result<Dictionary> {
    let setup: Function<Dictionary, Result<(), api::Error>> =
        Function::from_fn(move |_options: Dictionary| {
            let opts = CreateCommandOpts::builder()
                .desc("shows a greetings message")
                .nargs(CommandNArgs::Zero)
                .build();

            let greetings = |_args: CommandArgs| {
                let html = "<h1>Test</h1>";
                let out = from_read(html.as_bytes(), 20);
                print!("{out}");
            };

            api::create_user_command("Greetings", greetings, &opts)?;

            Ok(())
        });

    let mut buf = api::create_buf(false, true)?;
    let b = buf.clone();

    let win: Rc<RefCell<Option<Window>>> = Rc::default();
    let w = Rc::clone(&win);

    let window = move |_| -> Result<(), api::Error> {
        if w.borrow().is_some() {
            api::err_writeln("Devdocs window is already open");
            return Ok(());
        }

        let config = WindowConfig::builder()
            .relative(WindowRelativeTo::Editor)
            .border(WindowBorder::Rounded)
            .zindex(50)
            .title(WindowTitle::SimpleString(
                String::from("Devdocs.nvim").into(),
            ))
            .focusable(true)
            .style(WindowStyle::Minimal)
            .height(10)
            .width(40)
            .row(20)
            .col(20)
            .build();

        let mut win = w.borrow_mut();
        *win = Some(api::open_win(&b, true, &config)?);
        Ok(())
    };

    let opts = CreateCommandOpts::builder()
        .desc("Opens the Devdocs user interface")
        .nargs(CommandNArgs::Zero)
        .build();
    api::create_user_command("Devdocs", window, &opts)?;

    let close_window = move |_| {
        if win.borrow().is_none() {
            api::err_writeln("Devdocs window is already closed");
            return Ok(());
        }
        let win = win.borrow_mut().take().unwrap();
        win.close(false)
    };

    let opts = SetKeymapOpts::builder()
        .desc("Closes the Devdocs window")
        .callback(close_window)
        .nowait(true)
        .silent(true)
        .build();

    Buffer::set_keymap(&mut buf, Mode::Normal, "q", "", &opts)?;
    Buffer::set_keymap(&mut buf, Mode::Normal, "<Esc>", "q", &opts)?;

    let exports = Dictionary::from_iter([("setup", setup)]);
    Ok(exports)
}
