use salvo::prelude::*;

use crate::AppResult;


use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};




#[handler]
pub async fn index(res: &mut Response) -> AppResult<()> {


    // let email = Message::builder()
    // .from("yy <theyy.me@gmail.com>".parse().unwrap())
    // // .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
    // .to("Alan <alan@fpi.cc>".parse().unwrap())
    // .subject("Happy new year")
    // .header(ContentType::TEXT_PLAIN)
    // .body(String::from("Be happy!"))
    // .unwrap();

    // // 
    // let creds = Credentials::new("theyy.me@gmail.com".to_owned(), "!@#123Xicore".to_owned());

    // // Open a remote connection to gmail
    // let mailer = SmtpTransport::relay("smtp-relay.gmail.com")
    // .unwrap()
    // .credentials(creds)
    // .build();

    // // Send the email
    // match mailer.send(&email) {
    //     Ok(_) => println!("Email sent successfully!"),
    //     Err(e) => panic!("Could not send email: {e:?}"),
    // }




    res.render("Hello world");
    Ok(())
}
