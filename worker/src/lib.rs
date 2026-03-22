use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Deserialize)]
struct ContactRequest {
    name: String,
    email: String,
    subject: String,
    message: String,
    turnstile_token: String,
}

#[derive(Serialize)]
struct ContactResponse {
    success: bool,
    message: String,
}

#[derive(Deserialize)]
struct TurnstileResponse {
    success: bool,
}

fn cors_headers(origin: &str) -> Headers {
    let headers = Headers::new();
    let _ = headers.set("Access-Control-Allow-Origin", origin);
    let _ = headers.set("Access-Control-Allow-Methods", "POST, OPTIONS");
    let _ = headers.set("Access-Control-Allow-Headers", "Content-Type");
    let _ = headers.set("Access-Control-Max-Age", "86400");
    headers
}

fn json_response(body: &ContactResponse, status: u16, origin: &str) -> Result<Response> {
    let json = serde_json::to_string(body)?;
    let mut resp = Response::ok(json)?;
    *resp.headers_mut() = cors_headers(origin);
    let _ = resp.headers_mut().set("Content-Type", "application/json");
    if status != 200 {
        // worker crate doesn't have a direct status setter on Response::ok,
        // so we rebuild with the correct status
        let mut resp = Response::from_json(body)?.with_status(status);
        let headers = cors_headers(origin);
        for (key, val) in headers.entries() {
            let _ = resp.headers_mut().set(&key, &val);
        }
        return Ok(resp);
    }
    Ok(resp)
}

fn get_allowed_origin(env: &Env) -> String {
    env.var("ALLOWED_ORIGIN")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "*".to_string())
}

async fn verify_turnstile(token: &str, secret: &str) -> Result<bool> {
    let form = format!("secret={}&response={}", secret, token);

    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_body(Some(form.into()));

    let headers = Headers::new();
    headers.set("Content-Type", "application/x-www-form-urlencoded")?;
    init.with_headers(headers);

    let req = Request::new_with_init(
        "https://challenges.cloudflare.com/turnstile/v0/siteverify",
        &init,
    )?;

    let mut resp = Fetch::Request(req).send().await?;
    let result: TurnstileResponse = resp.json().await?;
    Ok(result.success)
}

async fn handle_contact(mut req: Request, env: &Env) -> Result<Response> {
    let origin = get_allowed_origin(env);

    let body: ContactRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => {
            return json_response(
                &ContactResponse {
                    success: false,
                    message: "Invalid request body.".into(),
                },
                400,
                &origin,
            );
        }
    };

    // Validate required fields
    if body.name.trim().is_empty()
        || body.email.trim().is_empty()
        || body.subject.trim().is_empty()
        || body.message.trim().is_empty()
    {
        return json_response(
            &ContactResponse {
                success: false,
                message: "All fields are required.".into(),
            },
            400,
            &origin,
        );
    }

    // Verify Turnstile token
    if let Ok(secret) = env.secret("TURNSTILE_SECRET_KEY") {
        let secret_str = secret.to_string();
        if !secret_str.is_empty() {
            match verify_turnstile(&body.turnstile_token, &secret_str).await {
                Ok(true) => {}
                Ok(false) => {
                    return json_response(
                        &ContactResponse {
                            success: false,
                            message: "Captcha verification failed.".into(),
                        },
                        400,
                        &origin,
                    );
                }
                Err(e) => {
                    console_error!("Turnstile verification error: {}", e);
                    // Fail open - don't block the message if Turnstile is down
                }
            }
        }
    }

    // Save to D1
    let db = env.d1("DB")?;
    let stmt = db.prepare(
        "INSERT INTO contact_messages (name, email, subject, message) VALUES (?1, ?2, ?3, ?4)",
    );
    if let Err(e) = stmt
        .bind(&[
            body.name.clone().into(),
            body.email.clone().into(),
            body.subject.clone().into(),
            body.message.clone().into(),
        ])?
        .run()
        .await
    {
        console_error!("D1 insert error: {}", e);
        return json_response(
            &ContactResponse {
                success: false,
                message: "Failed to save message. Please try again.".into(),
            },
            500,
            &origin,
        );
    }

    // Log for now - email sending via send_email binding can be added
    // once Email Routing is configured on the domain
    console_log!(
        "Contact form submission from {} <{}> - {}",
        body.name,
        body.email,
        body.subject
    );

    json_response(
        &ContactResponse {
            success: true,
            message: "Message sent! We'll be in touch soon.".into(),
        },
        200,
        &origin,
    )
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let origin = get_allowed_origin(&env);

    // Handle CORS preflight
    if req.method() == Method::Options {
        let mut resp = Response::empty()?.with_status(204);
        *resp.headers_mut() = cors_headers(&origin);
        return Ok(resp);
    }

    // Route: POST /api/v1/contact
    if req.method() == Method::Post && req.path() == "/api/v1/contact" {
        return handle_contact(req, &env).await;
    }

    // 404 for everything else
    Response::error("Not found", 404)
}
