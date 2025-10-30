use worker::*;

pub async fn route(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>API Docs</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.10.5/swagger-ui.css">
    <style>
        body {
            margin: 0;
            background: #0a0a0a;
        }

        /* Hide topbar */
        .swagger-ui .topbar { display: none; }

        /* Dark theme */
        .swagger-ui .scheme-container {
            background: transparent;
            color: #e5e7eb;
        }

        .swagger-ui {
            background: #0a0a0a;
        }

        .swagger-ui .wrapper { background: transparent; }

        .swagger-ui .opblock {
            background: #1f2937;
            border: 1px solid #374151;
        }

        .swagger-ui .opblock .opblock-summary {
            background: #111827;
            border-color: #374151;
        }

        .swagger-ui .opblock .opblock-summary:hover {
            background: #1f2937;
        }

        .swagger-ui .opblock.opblock-get .opblock-summary-method {
            background: #059669;
        }

        .swagger-ui .opblock.opblock-post .opblock-summary-method {
            background: #dc2626;
        }

        .swagger-ui .btn.authorize,
        .swagger-ui .btn.try-out__btn {
            background: #374151;
            border-color: #4b5563;
            color: #e5e7eb;
        }

        .swagger-ui input,
        .swagger-ui textarea,
        .swagger-ui select {
            background: #111827;
            border-color: #374151;
            color: #e5e7eb;
        }

        /* Text contrast fixes */
        .swagger-ui .opblock-description-wrapper p,
        .swagger-ui .opblock-description-wrapper,
        .swagger-ui .opblock-summary-description,
        .swagger-ui .parameter__name,
        .swagger-ui .parameter__type,
        .swagger-ui .response-col_status,
        .swagger-ui .response-col_description,
        .swagger-ui .model-title,
        .swagger-ui .model,
        .swagger-ui .info .title,
        .swagger-ui .info h1,
        .swagger-ui .info h2,
        .swagger-ui .info h3,
        .swagger-ui .renderedMarkdown p,
        .swagger-ui .renderedMarkdown code,
        .swagger-ui .opblock-section-header h4,
        .swagger-ui .opblock-section-header label,
        .swagger-ui .responses-inner h4,
        .swagger-ui .responses-inner h5 {
            color: #e5e7eb !important;
        }

        .swagger-ui .opblock .opblock-section-header {
            background: rgba(21, 30, 42);
        }

        .swagger-ui input[type=text] {
            background: transparent;
        }

        .swagger-ui .loading-container {
          background: transparent; /* light overlay */
        }

        .swagger-ui .loading {
            color: #f3f4f6 !important; /* dark text/spinner against light bg */
        }

        .swagger-ui .parameter__in,
        .swagger-ui .prop-type,
        .swagger-ui .prop-format,
        .swagger-ui .tab li {
            color: #9ca3af !important;
        }

        .swagger-ui .info .description,
        .swagger-ui .info p,
        .swagger-ui .parameters-col_description p,
        .swagger-ui .parameters-col_description {
            color: #d1d5db !important;
        }

        .swagger-ui table thead tr th,
        .swagger-ui table thead tr td {
            color: #f3f4f6 !important;
            border-color: #374151 !important;
        }

        .swagger-ui .opblock-body pre,
        .swagger-ui .microlight {
            background: #0f172a !important;
            color: #e5e7eb !important;
        }

        .swagger-ui .model-box,
        .swagger-ui .model-box-control {
            background: #111827 !important;
        }

        .swagger-ui .opblock-control-arrow {
            fill: #9ca3af !important;
        }

        .swagger-ui .wrapper a,
        .swagger-ui .wrapper a:visited {
            color: #e5e7eb;
        }

        .swagger-ui .wrapper .renderedMarkdown {
            color: rgba(229, 231, 235, 0.35);
        }

        .swagger-ui .wrapper .models-control {
            color: #f3f4f6;
        }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.10.5/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5.10.5/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = () => {
            window.ui = SwaggerUIBundle({
                url: '/api/openapi',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout",
                validatorUrl: null,
                tryItOutEnabled: true
            });
        };
    </script>
</body>
</html>"#;

    let mut resp = Response::from_html(html)?;
    resp.headers_mut().set("Cache-Control", "public, max-age=3600")?;
    Ok(resp)
}
