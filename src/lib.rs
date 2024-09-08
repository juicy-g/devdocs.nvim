use html2text::from_read;
use nvim_oxi::api::{self, opts::*, types::*, Buffer, Window};
use nvim_oxi::{print, Dictionary, Function};

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

    ////////////////////////////////////////////
    let mut buf = api::create_buf(false, true)?;

    use std::cell::RefCell;
    use std::rc::Rc;

    //I think I need to do the same thing for buf
    let win: Rc<RefCell<Option<Window>>> = Rc::default();
    let w = Rc::clone(&win);

    let close_window = move |_| {
        if win.borrow().is_none() {
            api::err_writeln("Devdocs window is already closed");
            return Ok(());
        }
        let win = win.borrow_mut().take().unwrap();
        win.close(false)
    };

    let close_keymap = |b: &mut Buffer| -> Result<(), api::Error> {
        let opts = SetKeymapOpts::builder()
            .desc("Closes the Devdocs window")
            .callback(close_window)
            .nowait(true)
            .silent(true)
            .build();

        //FIXME: This is not working. second line can't set a keymap because b goes out of scope.
        api::Buffer::set_keymap(b, Mode::Normal, "q", "", &opts)?;
        api::Buffer::set_keymap(b, Mode::Normal, "<Esc>", "", &opts)?;
        Ok(())
    };

    close_keymap(&mut buf).ok();
    ////////////////////////////////////////////

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
        *win = Some(api::open_win(&buf, true, &config)?);
        Ok(())
    };

    let opts = CreateCommandOpts::builder()
        .desc("Opens and closes the Devdocs user interface")
        .nargs(CommandNArgs::Zero)
        .build();
    api::create_user_command("Devdocs", window, &opts)?;

    let exports = Dictionary::from_iter([("setup", setup)]);
    Ok(exports)
}
