// use html2text::from_read;
use nvim_oxi::api::{self, opts::*, types::*, Buffer, Window};
use nvim_oxi::{print, Dictionary, Function};
use std::cell::RefCell;
use std::rc::Rc;
use validator::Validate;

mod config;
use config::{Options, OptionsOpt};

#[nvim_oxi::plugin]
fn devdocs() -> nvim_oxi::Result<Dictionary> {
    let setup: Function<OptionsOpt, Result<(), api::Error>> =
        Function::from_fn(|mut opts: OptionsOpt| {
            // validate options from setup function
            match opts.validate() {
                Ok(_) => (),
                Err(e) => {
                    for values in e.field_errors().values() {
                        for error in values.iter() {
                            use std::borrow::Borrow;
                            match error.code.borrow() {
                                "url" => {
                                    // replace invalid url with the default url
                                    opts.url = Default::default();
                                    print!("{}", error.message.to_owned().unwrap());
                                }
                                &_ => todo!(),
                            }
                        }
                    }
                }
            };

            let mut options = Options {
                ..Default::default()
            };
            options.merge(opts);

            // print!("{options:?}");
            // let opts = CreateCommandOpts::builder()
            //     .desc("shows a greetings message")
            //     .nargs(CommandNArgs::Zero)
            //     .build();

            // let greetings = |_args: CommandArgs| {
            //     let html = "<h1>Test</h1>";
            //     let out = from_read(html.as_bytes(), 20);
            //     print!("{out}");
            // };

            // api::create_user_command("Greetings", greetings, &opts)?;
            Ok(())
        });

    let opts = CreateAugroupOpts::builder().clear(false).build();
    api::create_augroup("Devdocs", &opts)?;

    //Setup buffer
    let mut buf = api::create_buf(false, true)?;
    api::Buffer::set_lines(
        &mut buf,
        ..1,
        true,
        [" Press q or <Esc> to close this window."],
    )?;
    api::Buffer::add_highlight(&mut buf, 0, "Comment", 0, 0..)?;
    let b = buf.clone();

    let win: Rc<RefCell<Option<Window>>> = Rc::default();
    let w = Rc::clone(&win);

    let open_window = move |_| -> Result<(), api::Error> {
        if w.borrow().is_some() {
            api::err_writeln("Devdocs window is already open");
            return Ok(());
        }

        let opts = OptionOpts::builder()
            .scope(api::opts::OptionScope::Global)
            .build();

        let lines: u32 = api::get_option_value("lines", &opts).unwrap();
        let height: f32 = lines as f32 * 0.8;

        let columns: u32 = api::get_option_value("columns", &opts).unwrap();
        let width: f32 = columns as f32 * 0.8;

        let config = WindowConfig::builder()
            .relative(WindowRelativeTo::Editor)
            .border(WindowBorder::Rounded)
            .zindex(50)
            .title(WindowTitle::SimpleString(
                String::from("Devdocs.nvim").into(),
            ))
            .focusable(true)
            .style(WindowStyle::Minimal)
            .height(height.floor() as u32)
            .width(width.floor() as u32)
            .row(((lines as f32 - height) / 2.0).floor() as u32)
            .col(((columns as f32 - width) / 2.0).floor() as u32)
            .build();

        let mut win = w.borrow_mut();
        *win = Some(api::open_win(&b, true, &config)?);
        Ok(())
    };

    let opts = CreateCommandOpts::builder()
        .desc("Opens the Devdocs user interface")
        .nargs(CommandNArgs::Zero)
        .build();
    api::create_user_command("Devdocs", open_window, &opts)?;

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
