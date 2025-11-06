use worker::*;

pub async fn handler(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>404 - You're Lost!</title>
  <style>
    :root {
      --color-primary: #7c3aed;
      --color-text-dark: #1f2937;
      --color-text-light: #6b7280;
      --color-bg: #f9fafb;
      --color-white: #ffffff;
      --radius: 8px;
      --shadow: 0 1px 3px rgba(0,0,0,0.1), 0 1px 2px rgba(0,0,0,0.06);
    }
    
    * { margin: 0; padding: 0; box-sizing: border-box; }
    
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
      background: var(--color-bg);
      color: var(--color-text-dark);
      line-height: 1.6;
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 16px;
    }
    
    .container {
      background: var(--color-white);
      border-radius: var(--radius);
      box-shadow: var(--shadow);
      max-width: 620px;
      width: 100%;
      padding: clamp(24px, 6vw, 40px);
    }
    
    h1 {
      font-size: clamp(22px, 5vw, 32px);
      font-weight: 700;
      margin-bottom: 24px;
      letter-spacing: -0.5px;
      line-height: 1.15;
    }
    
    .error-code {
      font-size: clamp(18px, 4vw, 22px);
      color: var(--color-text-light);
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 1px;
      margin-bottom: 24px;
      padding-bottom: 16px;
      border-bottom: 1px solid rgba(0, 0, 0, 0.05);
      text-align: center;
    }
    
    .gospel {
      margin: 0;
    }
    
    .gospel p {
      margin-bottom: 16px;
      font-size: 16px;
      line-height: 1.75;
      color: var(--color-text-light);
    }
    
    .gospel p:first-of-type {
      margin-top: 8px;
    }
    
    .gospel strong {
      color: var(--color-text-dark);
      font-weight: 600;
    }
    
    .quote {
      padding: 20px;
      margin: 24px 0;
      background: linear-gradient(135deg, #f3e8ff 0%, #faf5ff 100%);
      border-left: 4px solid var(--color-primary);
      border-radius: 4px;
      color: var(--color-text-dark);
      line-height: 1.8;
    }
    
    .quote em {
      font-style: italic;
    }
    
    .cta {
      margin-top: 24px;
      text-align: center;
    }
    
    .cta a {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 8px 16px;
      background: var(--color-white);
      color: var(--color-primary);
      text-decoration: none;
      border-radius: var(--radius);
      font-weight: 600;
      font-size: 16px;
      transition: all 150ms ease;
      border: 2px solid var(--color-primary);
    }
    
    .cta a:hover {
      background: var(--color-primary);
      color: var(--color-white);
      transform: translateY(-2px);
      box-shadow: 0 10px 20px rgba(124, 58, 237, 0.2);
    }
    
    .cta a:focus-visible {
      outline: 2px solid var(--color-primary);
      outline-offset: 2px;
    }
    
    @media (max-width: 640px) {
      .cta a {
        width: 100%;
        justify-content: center;
      }
    }
  </style>
</head>
<body>
  <div class="container">
    <div class="error-code">404 - Page Not Found</div>
    <h1>You're Lost!<br>Get Found with the Gospel ✝️</h1>
    
    <div class="gospel">
      <p>Gospel means good news! The bad news is we have all sinned and deserve the wrath to come. But Jesus the Messiah died for our sins, was buried, and then raised on the third day, according to the scriptures. He ascended into heaven and right now is seated at the Father's right hand.</p>
      
      <div class="quote">
        Jesus said, <strong><em>"I am the way, and the truth, and the life. No one comes to the Father except through me. The time is fulfilled, and the kingdom of God is at hand; repent and believe in the gospel."</em></strong>
      </div>
      
      <div class="cta">
        <a href="https://read.lsbible.org/?q=John">📖 Read the Gospel of John</a>
      </div>
    </div>
  </div>
</body>
</html>"#;

    Ok(Response::from_html(html)?.with_status(404))
}
