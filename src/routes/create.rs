use worker::*;
use qrcode::{QrCode, render::svg};

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#x27;")
}

fn generate_short_id() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let timestamp = Date::now().as_millis() as u64;
    let mut id = String::new();
    let mut num = timestamp;
    for _ in 0..6 {
        id.push(CHARSET[(num % CHARSET.len() as u64) as usize] as char);
        num /= CHARSET.len() as u64;
    }
    id
}

pub async fn get_handler(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>EdgeLink - Create URL Redirect</title>
    <style>
        body { font-family: system-ui; max-width: 600px; margin: 50px auto; padding: 20px; background: #f9f9f9; }
        .container { background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        h1 { color: #24292f; margin: 0 0 25px 0; font-size: 24px; }
        .field { margin-bottom: 20px; }
        .field label { display: block; font-weight: 600; color: #333; margin-bottom: 6px; font-size: 14px; }
        input { width: 100%; box-sizing: border-box; padding: 8px 12px; border: 1px solid #d0d7de; border-radius: 4px; font-size: 14px; font-family: system-ui; }
        input:focus { outline: none; border-color: #0969da; box-shadow: 0 0 0 3px rgba(9,105,218,0.1); }
        button { width: 100%; padding: 10px; background: #0969da; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 600; transition: background 0.2s; }
        button:hover { background: #0860ca; }
        button:active { background: #0757ba; }
    </style>
</head>
<body>
    <div id="container" class="container">
        <h1 id="title">⚡ Create URL Redirect and QR Code</h1>
        <form id="create-form" method="POST" action="/create">
            <div id="name-field" class="field">
                <label id="name-label" for="name">Custom name (optional)</label>
                <input id="name" type="text" name="name" placeholder="Leave blank for auto-generated" maxlength="20">
            </div>
            <div id="url-field" class="field">
                <label id="url-label" for="url">URL</label>
                <input id="url" type="text" name="url" placeholder="example.com or https://example.com" required>
            </div>
            <button id="submit-btn" type="submit">Generate QR Code</button>
        </form>
    </div>
</body>
</html>"#;

    Response::from_html(html)
}

pub async fn post_handler(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let email = req.headers()
        .get("cf-access-authenticated-user-email")
        .ok()
        .flatten()
        .unwrap_or_else(|| "local@example.com".to_string());

    let form = match req.form_data().await {
        Ok(form) => form,
        Err(_) => return render_error("Invalid form data"),
    };

    let url = match form.get("url") {
        Some(worker::FormEntry::Field(url)) => url,
        _ => return render_error("URL is required"),
    };

    // Get optional custom name (auto-normalize: lowercase, spaces to hyphens, remove invalid chars)
    let custom_name = match form.get("name") {
        Some(worker::FormEntry::Field(name)) => {
            let normalized: String = name.trim()
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
                .collect();
            if normalized.is_empty() {
                None
            } else {
                Some(normalized)
            }
        },
        _ => None,
    };

    if url.is_empty() {
        return render_error("URL is required");
    }

    // Auto-prepend https:// if no protocol specified
    let url = if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url
    };

    // Validate URL format and domain structure
    let parsed_url = match Url::parse(&url) {
        Ok(u) => u,
        Err(_) => return render_error("Invalid URL format"),
    };

    // Ensure URL has a valid host with at least a domain extension
    match parsed_url.host_str() {
        Some(host) => {
            if !host.contains('.') {
                return render_error("URL must include a domain extension (e.g., .com, .org)");
            }
        },
        None => return render_error("URL must include a valid domain"),
    }

    let kv = ctx.env.kv("edgelink")?;

    // Determine short ID: use custom name if provided and valid, otherwise generate
    let short_id = if let Some(name) = custom_name {
        // Validate custom name length
        if name.len() < 2 || name.len() > 20 {
            return render_error("Custom name must be 2-20 characters");
        }

        // Check if name already exists
        if kv.get(&name).text().await?.is_some() {
            return render_error(&format!("Name '{}' is already taken", name));
        }

        name
    } else {
        // Auto-generate ID with collision detection
        loop {
            let id = generate_short_id();
            if kv.get(&id).text().await?.is_none() {
                break id;
            }
        }
    };

    kv.put(&short_id, &url)?
        .metadata(serde_json::json!({
            "created_by": email,
            "created_at": Date::now().to_string(),
        }))?
        .execute()
        .await?;

    let host = req.headers()
        .get("host")
        .ok()
        .flatten()
        .unwrap_or_else(|| "localhost:8787".to_string());
    let protocol = if host.contains("localhost") { "http" } else { "https" };

    let short_url = format!("{}://{}/{}", protocol, host, short_id);
    let code = QrCode::new(short_url.as_bytes())
        .map_err(|_| Error::from("Failed to generate QR code"))?;

    let qr_svg = code.render::<svg::Color>()
        .min_dimensions(300, 300)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    render_success(&short_url, &url, &qr_svg)
}

fn render_error(message: &str) -> Result<Response> {
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>EdgeLink - Create URL Redirect</title>
    <style>
        body {{ font-family: system-ui; max-width: 600px; margin: 50px auto; padding: 20px; background: #f9f9f9; }}
        .container {{ background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        h1 {{ color: #24292f; margin: 0 0 25px 0; font-size: 24px; }}
        .field {{ margin-bottom: 20px; }}
        .field label {{ display: block; font-weight: 600; color: #333; margin-bottom: 6px; font-size: 14px; }}
        input {{ width: 100%; box-sizing: border-box; padding: 8px 12px; border: 1px solid #d0d7de; border-radius: 4px; font-size: 14px; font-family: system-ui; }}
        input:focus {{ outline: none; border-color: #0969da; box-shadow: 0 0 0 3px rgba(9,105,218,0.1); }}
        button {{ width: 100%; padding: 10px; background: #0969da; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 600; transition: background 0.2s; }}
        button:hover {{ background: #0860ca; }}
        button:active {{ background: #0757ba; }}
        .error {{ background: #ffebe9; border: 1px solid #ff8182; color: #d1242f; padding: 12px; border-radius: 4px; margin-bottom: 20px; font-size: 14px; }}
    </style>
</head>
<body>
    <div id="container" class="container">
        <h1 id="title">⚡ Create URL Redirect and QR Code</h1>
        <div id="error-message" class="error">{}</div>
        <form id="create-form" method="POST" action="/create">
            <div id="name-field" class="field">
                <label id="name-label" for="name">Custom name (optional)</label>
                <input id="name" type="text" name="name" placeholder="Leave blank for auto-generated" maxlength="20">
            </div>
            <div id="url-field" class="field">
                <label id="url-label" for="url">URL</label>
                <input id="url" type="text" name="url" placeholder="example.com or https://example.com" required>
            </div>
            <button id="submit-btn" type="submit">Generate QR Code</button>
        </form>
    </div>
</body>
</html>"#, html_escape(message));

    Response::from_html(html)
}

fn render_success(short_url: &str, destination_url: &str, qr_svg: &str) -> Result<Response> {
    let escaped_short = html_escape(short_url);
    let escaped_dest = html_escape(destination_url);

    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>EdgeLink - URL Created</title>
    <style>
        body {{ font-family: system-ui; max-width: 800px; margin: 50px auto; padding: 20px; background: #f9f9f9; }}
        .container {{ display: flex; gap: 30px; align-items: flex-start; margin: 30px 0; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .info {{ flex: 1; min-width: 0; }}
        .field {{ margin-bottom: 20px; }}
        .field label {{ display: block; font-weight: 600; color: #333; margin-bottom: 6px; font-size: 14px; }}
        .code-block {{ position: relative; background: #f6f8fa; border: 1px solid #d0d7de; border-radius: 4px; padding: 8px 40px 8px 12px; }}
        .code-block code {{ font-family: monospace; font-size: 13px; color: #24292f; display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }}
        .code-block a {{ color: #0969da; text-decoration: none; }}
        .code-block a:hover {{ text-decoration: underline; }}
        .copy-btn {{ position: absolute; top: 0; right: 0; bottom: 0; padding: 0 8px; background: white; border: 1px solid #d0d7de; border-radius: 4px; cursor: pointer; color: #57606a; transition: all 0.2s; display: flex; align-items: center; justify-content: center; }}
        .copy-btn:hover {{ background: #f6f8fa; border-color: #8c959f; }}
        .copy-btn svg {{ width: 16px; height: 16px; fill: currentColor; display: block; }}
        .copy-btn.copied {{ background: #28a745; border-color: #28a745; color: white; }}
        .qr-container .copy-btn {{ background: rgba(255,255,255,0.9); border: 1px solid #d0d7de; color: #57606a; bottom: auto; padding: 6px; }}
        .qr-container .copy-btn:hover {{ background: white; border-color: #8c959f; }}
        .qr-container .copy-btn.copied {{ background: #28a745; border-color: #28a745; color: white; }}
        .qr-container .copy-svg-btn {{ top: auto; bottom: 0; padding: 0; width: 28px; height: 28px; font-family: monospace; font-size: 11px; font-weight: bold; display: flex; align-items: center; justify-content: center; }}
        .qr-container {{ flex-shrink: 0; position: relative; width: 200px; height: 200px; }}
        .qr-container .qr-code {{ width: 100%; height: 100%; border: 1px solid #d0d7de; box-sizing: border-box; border-radius: 4px; background: white; padding: 15px; }}
        .qr-container .qr-code svg {{ width: 100%; height: 100%; display: block; }}
        .actions {{ margin-top: 25px; padding-top: 20px; border-top: 1px solid #eee; }}
        .actions a {{ color: #0066cc; text-decoration: none; font-weight: 500; }}
        .actions a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div id="container" class="container">
        <div id="info" class="info">
            <div id="url-field" class="field">
                <label id="url-label">URL</label>
                <div id="url-block" class="code-block">
                    <code><a id="url-link" href="{}" target="_blank">{}</a></code>
                    <button id="copy-url-btn" class="copy-btn" onclick="copy('{}', this)" aria-label="Copy URL" title="Click to copy"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M14 1.5H6l-.5.5v2.5h1v-2h7v7h-2v1H14l.5-.5V2l-.5-.5z"></path><path d="M2 5.5l-.5.5v8l.5.5h8l.5-.5V6l-.5-.5H2zm7.5 8h-7v-7h7v7z"></path></svg></button>
                </div>
            </div>
            <div id="target-field" class="field">
                <label id="target-label">Target</label>
                <div id="target-block" class="code-block">
                    <code><a id="target-link" href="{}" target="_blank">{}</a></code>
                    <button id="copy-target-btn" class="copy-btn" onclick="copy('{}', this)" aria-label="Copy target" title="Click to copy"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M14 1.5H6l-.5.5v2.5h1v-2h7v7h-2v1H14l.5-.5V2l-.5-.5z"></path><path d="M2 5.5l-.5.5v8l.5.5h8l.5-.5V6l-.5-.5H2zm7.5 8h-7v-7h7v7z"></path></svg></button>
                </div>
            </div>
            <div id="actions" class="actions">
                <a id="create-another" href="/create">← Create another</a>
            </div>
        </div>
        <div id="qr-container" class="qr-container">
            <div id="qr-code" class="qr-code">{}</div>
            <button id="copy-qr-btn" class="copy-btn" onclick="copyQR(this)" aria-label="Copy QR code as PNG" title="Copy as PNG image"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect x="2" y="2" width="12" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/><circle cx="5" cy="5.5" r="1.2" fill="currentColor"/><path d="M2 9.5L5.5 6.5L8 8.5L11.5 5L14 7.5V11c0 .55-.45 1-1 1H3c-.55 0-1-.45-1-1V9.5z" fill="currentColor"/></svg></button>
            <button id="copy-svg-btn" class="copy-btn copy-svg-btn" onclick="copySVG(this)" aria-label="Copy SVG code" title="Copy SVG code">&lt;/&gt;</button>
        </div>
    </div>
    <script>
        function copy(text, btn) {{
            navigator.clipboard.writeText(text).then(() => {{
                btn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M13.5 2.5l-8 8-3-3-1 1 4 4 9-9z"></path></svg>';
                btn.classList.add('copied');
                setTimeout(() => {{
                    btn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M14 1.5H6l-.5.5v2.5h1v-2h7v7h-2v1H14l.5-.5V2l-.5-.5z"></path><path d="M2 5.5l-.5.5v8l.5.5h8l.5-.5V6l-.5-.5H2zm7.5 8h-7v-7h7v7z"></path></svg>';
                    btn.classList.remove('copied');
                }}, 2000);
            }});
        }}

        function copyQR(btn) {{
            const svg = document.querySelector('#qr-code svg');
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');
            const img = new Image();
            
            const svgData = new XMLSerializer().serializeToString(svg);
            const svgBlob = new Blob([svgData], {{type: 'image/svg+xml;charset=utf-8'}});
            const url = URL.createObjectURL(svgBlob);
            
            img.onload = () => {{
                canvas.width = img.width;
                canvas.height = img.height;
                ctx.fillStyle = 'white';
                ctx.fillRect(0, 0, canvas.width, canvas.height);
                ctx.drawImage(img, 0, 0);
                
                canvas.toBlob(blob => {{
                    navigator.clipboard.write([
                        new ClipboardItem({{'image/png': blob}})
                    ]).then(() => {{
                        btn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M13.5 2.5l-8 8-3-3-1 1 4 4 9-9z"></path></svg>';
                        btn.classList.add('copied');
                        setTimeout(() => {{
                            btn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M14 1.5H6l-.5.5v2.5h1v-2h7v7h-2v1H14l.5-.5V2l-.5-.5z"></path><path d="M2 5.5l-.5.5v8l.5.5h8l.5-.5V6l-.5-.5H2zm7.5 8h-7v-7h7v7z"></path></svg>';
                            btn.classList.remove('copied');
                        }}, 2000);
                    }});
                }});
                
                URL.revokeObjectURL(url);
            }};
            
            img.src = url;
        }}

        function copySVG(btn) {{
            const svg = document.querySelector('#qr-code svg');
            const svgData = new XMLSerializer().serializeToString(svg);
            
            navigator.clipboard.writeText(svgData).then(() => {{
                const originalText = btn.innerHTML;
                btn.innerHTML = '✓';
                btn.classList.add('copied');
                setTimeout(() => {{
                    btn.innerHTML = originalText;
                    btn.classList.remove('copied');
                }}, 2000);
            }});
        }}
    </script>
</body>
</html>"#, escaped_short, escaped_short, escaped_short, escaped_dest, escaped_dest, escaped_dest, qr_svg);

    Response::from_html(html)
}
