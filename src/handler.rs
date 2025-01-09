use actix_web::web;

use crate::api::file::{
    create_file, delete_file, get_file, get_files, get_files_by_directory, update_file,
};
use crate::api::user::{auth_login, create_user, delete_user, get_user_me, update_user_me};

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/v1")
        .service(get_file)
        .service(get_files)
        .service(get_files_by_directory)
        .service(create_file)
        .service(update_file)
        .service(delete_file)
        .service(auth_login)
        .service(create_user)
        .service(update_user_me)
        .service(get_user_me)
        .service(delete_user);

    conf.service(scope);
}
