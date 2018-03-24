use super::http::response::{create_response, extend_response};
use super::router::response::extender::StaticResponseExtender;
use super::state::{FromState, State, StateData};
use hyper;
use mime;
use mime_guess;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

pub struct StaticFileHandler {
    root: PathBuf,
}

impl StaticFileHandler {
    pub fn new(root: PathBuf) -> StaticFileHandler {
        StaticFileHandler { root }
    }

    pub fn static_page(&self, state: State) -> (State, hyper::Response) {
        let path = {
            let mut path = self.root.clone();
            for component in &FilePathExtractor::borrow_from(&state).parts {
                path.push(component);
            }
            path
        };
        let response = path.metadata()
            .and_then(|meta| {
                let mut contents = Vec::with_capacity(meta.len() as usize);
                fs::File::open(&path).and_then(|mut f| f.read_to_end(&mut contents))?;
                Ok(contents)
            })
            .map(|contents| {
                let extension = path.extension()
                    .and_then(|p| p.to_str())
                    .unwrap_or_default();
                let mime_type = mime_guess::get_mime_type(extension);
                create_response(&state, hyper::StatusCode::Ok, Some((contents, mime_type)))
            })
            .unwrap_or_else(|err| error_response(&state, err));
        (state, response)
    }
}

fn error_response(state: &State, e: io::Error) -> hyper::Response {
    let status = match e.kind() {
        io::ErrorKind::NotFound => hyper::StatusCode::NotFound,
        io::ErrorKind::PermissionDenied => hyper::StatusCode::Forbidden,
        _ => hyper::StatusCode::InternalServerError,
    };
    create_response(
        &state,
        status,
        Some((format!("{}", status).into_bytes(), mime::TEXT_PLAIN)),
    )
}

#[derive(Debug, Deserialize)]
pub struct FilePathExtractor {
    #[serde(rename = "*")]
    parts: Vec<String>,
}

impl StateData for FilePathExtractor {}

// This is a copy-paste of the #derive implementation. It would be nice not to need to do this, but
// that would require depending on gotham_derive, which would mean this implementation would need to
// live somewhere other than the gotham crate.
impl StaticResponseExtender for FilePathExtractor {
    fn extend(state: &mut State, res: &mut hyper::Response) {
        extend_response(state, res, ::hyper::StatusCode::BadRequest, None);
    }
}
