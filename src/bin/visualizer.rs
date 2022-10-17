use clap::Parser;
use performance_data::VISUALIZATION_DATA;
use tide::http::mime;
use tide::{Response, StatusCode};

#[derive(Clone, Parser, Debug)]
#[clap(author, about, long_about = None)]
#[clap(name = "performance-data-visualizer")]
#[clap(version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("VERGEN_GIT_SHA_SHORT"), ")"))]
struct Args {
    /// Directory which contains run data to be visualized.
    #[clap(short, long, value_parser)]
    run_directory: Vec<String>,

    /// Port number on which to listen for connections.
    #[clap(short, long, value_parser, default_value_t = 8080)]
    port_number: u64,
}

fn create_response(http_code: tide::StatusCode, body: &str, content_type: mime::Mime) -> Response {
    Response::builder(http_code).content_type(content_type).body(body.clone()).build()
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    tide::log::start();

    let dirs: Vec<String> = args.run_directory;
    for dir in dirs {
        let name;
        name = VISUALIZATION_DATA.lock().unwrap().init_visualizers(dir.to_owned() + "/").unwrap();
        VISUALIZATION_DATA.lock().unwrap().unpack_data(name).unwrap();
    }

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());

    app.at("/").get(|_| async move {
        let html = include_str!("html_files/index.html");
        Ok(create_response(StatusCode::Ok, html, mime::HTML))
    });
    /* Serve JavaScript files */
    app.at("/html_files/:name").get(|req: tide::Request<()>| async move {
        let name = req.param("name").unwrap();
        let file;
        let mut file_type = mime::JAVASCRIPT;
        match name {
            "index.css" => {
                file = include_str!("html_files/index.css");
                file_type = mime::CSS;
            },
            "index.js" => file = include_str!("html_files/js/index.js"),
            "plotly-2.14.0.min.js" => file = include_str!("html_files/plotly-2.14.0.min.js"),
            _ => return Ok(create_response(
                    StatusCode::Ok,
                    VISUALIZATION_DATA.lock().unwrap().get_js_file((&name).to_string())?,
                    mime::JAVASCRIPT)),
        }
        Ok(create_response(StatusCode::Ok, file, file_type))
    });
    /* Data to visualize */
    app.at("/visualize/:name").get(|req: tide::Request<()>| async move {
        let api_name = req.param("name").unwrap();
        let query = req.url().query().ok_or("Error unwrapping query");
        let data;
        match api_name {
            "get" => data = VISUALIZATION_DATA.lock().unwrap().get_run_names(),
            _ => data = VISUALIZATION_DATA.lock().unwrap().get_data(&api_name, query.unwrap().to_string()),
        }
        match data {
            Ok(value) => Ok(create_response(StatusCode::Ok, &value, mime::JAVASCRIPT)),
            Err(e) => panic!("{:#?}", e),
        }
    });
    app.listen(format!("127.0.0.1:{}", args.port_number)).await?;
    Ok(())
}