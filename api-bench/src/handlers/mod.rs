use std::fs;
use std::time::Instant;

use axum::{extract::Query, response::IntoResponse};
use serde::Deserialize;

use crate::models::{BatchReport, BlogPost, Order, Product, SimpleUser};

#[derive(Debug, Deserialize)]
pub struct BenchmarkQuery {
    sample: Option<i32>,
    report: Option<bool>,
    json_type: Option<String>,
    clear: Option<bool>,
    delete: Option<String>,
}

pub async fn benchmark_handler(Query(query): Query<BenchmarkQuery>) -> impl IntoResponse {
    if let Some(ids_to_delete) = query.delete.clone() {
        for id in ids_to_delete.split(',') {
            let id = id.trim();
            if !id.is_empty() {
                let _ = fs::remove_file(format!("api-bench/reports/{}.html", id));
                let _ = fs::remove_file(format!("api-bench/reports/{}.json", id));
            }
        }
        return (
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            generate_index_html(),
        );
    }

    if query.clear.unwrap_or(false) {
        let _ = fs::remove_dir_all("api-bench/reports");
        let _ = fs::create_dir_all("api-bench/reports");
        return (
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            generate_index_html(),
        );
    }

    let sample_size = query.sample.unwrap_or(1000);
    let show_report = query.report.unwrap_or(false);
    let json_type = query.json_type.unwrap_or_else(|| "all".to_string());

    let results = match json_type.as_str() {
        "simple_user" => vec![run_simple_user_benchmark(sample_size).await],
        "batch_users" => vec![run_batch_users_benchmark(sample_size).await],
        "product" => vec![run_product_benchmark(sample_size).await],
        "order" => vec![run_order_benchmark(sample_size).await],
        "blog_post" => vec![run_blog_post_benchmark(sample_size).await],
        "all" => vec![
            run_simple_user_benchmark(sample_size).await,
            run_batch_users_benchmark(sample_size).await,
            run_product_benchmark(sample_size).await,
            run_order_benchmark(sample_size).await,
            run_blog_post_benchmark(sample_size).await,
        ],
        _ => vec![run_simple_user_benchmark(sample_size).await],
    };

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let report_id = format!("{}_{}", sample_size, timestamp);

    let _ = fs::create_dir_all("api-bench/reports");
    let json_content = serde_json::to_string(&results).unwrap_or_default();
    let _ = fs::write(
        format!("api-bench/reports/{}.json", report_id),
        &json_content,
    );

    let html_content = generate_report_html(&results, sample_size, &json_type, &report_id);
    let _ = fs::write(
        format!("api-bench/reports/{}.html", report_id),
        &html_content,
    );

    if show_report {
        (
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            html_content,
        )
    } else {
        let json = serde_json::to_string(&results).unwrap_or_default();
        (
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            json,
        )
    }
}

pub async fn index_handler() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        generate_index_html(),
    )
}

pub async fn report_handler(Query(query): Query<BenchmarkQuery>) -> impl IntoResponse {
    let sample = query.sample.unwrap_or(0);

    if sample > 0
        && let Ok(entries) = fs::read_dir("api-bench/reports")
    {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(name) = path.file_stem() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with(&format!("{}_", sample))
                    && let Ok(content) = fs::read_to_string(&path)
                {
                    return (
                        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
                        content,
                    );
                }
            }
        }
    }

    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        generate_index_html(),
    )
}

fn get_reports_list() -> Vec<(String, String, String)> {
    let mut reports = Vec::new();

    if let Ok(entries) = fs::read_dir("api-bench/reports") {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "html").unwrap_or(false)
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                && let Ok(metadata) = entry.metadata()
                && let Ok(modified) = metadata.modified()
            {
                let datetime: chrono::DateTime<chrono::Local> = modified.into();
                let date = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                let parts: Vec<&str> = name.split('_').collect();
                let sample = parts.first().unwrap_or(&"?").to_string();
                reports.push((name.to_string(), sample, date));
            }
        }
    }

    reports.sort_by(|a, b| b.2.cmp(&a.2));
    reports
}

fn generate_index_html() -> String {
    let reports = get_reports_list();

    let reports_table = if reports.is_empty() {
        String::from(
            r#"<div class="empty"><div class="empty-icon">📊</div><p>No benchmark reports yet.</p></div>"#,
        )
    } else {
        let mut html = String::from(
            r#"<table><thead><tr><th><input type="checkbox" id="selectAll" onchange="toggleAll(this)"></th><th>Sample</th><th>Date</th><th>Action</th></tr></thead><tbody>"#,
        );
        for (id, sample, date) in &reports {
            html.push_str(&format!(r#"<tr><td><input type="checkbox" class="report-check" value="{}"></td><td><strong>{}</strong></td><td>{}</td><td><a href="/report?sample={}" class="report-link">View</a></td></tr>"#, id, sample, date, sample));
        }
        html.push_str("</tbody></table>");
        html
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>FastSerial Benchmark</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'Segoe UI', Arial, sans-serif; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; padding: 40px; }}
        .container {{ max-width: 900px; margin: 0 auto; }}
        h1 {{ color: white; text-align: center; margin-bottom: 10px; }}
        .subtitle {{ color: #e0e0e0; text-align: center; margin-bottom: 30px; }}
        .card {{ background: white; padding: 25px; border-radius: 12px; margin-bottom: 20px; }}
        .controls {{ display: flex; gap: 15px; flex-wrap: wrap; margin-bottom: 20px; }}
        .control-group {{ display: flex; align-items: center; gap: 10px; }}
        .control-group label {{ font-weight: 600; color: #333; }}
        .control-group select {{ padding: 10px; border: 2px solid #ddd; border-radius: 8px; }}
        .btn {{ padding: 12px 24px; background: #667eea; color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: 600; text-decoration: none; }}
        .btn:hover {{ background: #5568d3; }}
        .btn-red {{ background: #e74c3c; }}
        .btn-red:hover {{ background: #c0392b; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 15px; text-align: left; border-bottom: 1px solid #eee; }}
        th {{ background: #667eea; color: white; }}
        .report-link {{ color: #667eea; text-decoration: none; font-weight: 600; }}
        .empty {{ text-align: center; color: #666; padding: 40px; }}
        .legend {{ background: white; padding: 15px; border-radius: 8px; margin-bottom: 20px; display: flex; gap: 30px; justify-content: center; }}
        .legend span {{ display: flex; align-items: center; gap: 8px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>FastSerial Benchmark Reports</h1>
        <p class="subtitle">Saved benchmark results</p>
        
        <div class="legend" style="background:white;padding:15px;border-radius:8px;margin-bottom:20px;display:flex;gap:30px;justify-content:center;">
            <span><strong>fs</strong> = fastserial (our library)</span>
            <span><strong>sj</strong> = serde_json (comparison)</span>
        </div>
        
        <div class="card">
            <h3 style="margin-bottom:15px;">Run New Benchmark</h3>
            <div class="controls">
                <div class="control-group">
                    <label>Sample:</label>
                    <select id="sample">
                        <option value="10">10</option>
                        <option value="100">100</option>
                        <option value="1000" selected>1000</option>
                        <option value="5000">5000</option>
                    </select>
                </div>
                <div class="control-group">
                    <label>JSON:</label>
                    <select id="jsonType">
                        <option value="all">All Tests</option>
                        <option value="simple_user">Simple User</option>
                        <option value="batch_users">Batch Users</option>
                        <option value="product">Product</option>
                        <option value="order">Order</option>
                        <option value="blog_post">Blog Post</option>
                    </select>
                </div>
                <button class="btn" onclick="runBenchmark()">Run & Save</button>
                <button class="btn btn-red" onclick="deleteSelected()">Delete Selected</button>
            </div>
        </div>
        
        <div class="card">
            <h3>Saved Reports</h3>
            {}
        </div>
    </div>
    
    <script>
        function runBenchmark() {{
            const sample = document.getElementById('sample').value;
            const jsonType = document.getElementById('jsonType').value;
            window.location.href = '/bench?sample=' + sample + '&json_type=' + jsonType + '&report=true';
        }}
        function toggleAll(source) {{
            document.querySelectorAll('.report-check').forEach(cb => cb.checked = source.checked);
        }}
        function deleteSelected() {{
            const checkboxes = document.querySelectorAll('.report-check:checked');
            const ids = Array.from(checkboxes).map(cb => cb.value).join(',');
            if (ids && confirm('Delete ' + checkboxes.length + ' report(s)?')) {{
                window.location.href = '/bench?delete=' + ids;
            }}
        }}
    </script>
</body>
</html>"#,
        reports_table
    )
}

fn generate_report_html(
    results: &[BatchReport],
    sample_size: i32,
    json_type: &str,
    report_id: &str,
) -> String {
    let avg_encode = if results.is_empty() {
        1.0
    } else {
        results.iter().map(|r| r.encode_speedup).sum::<f64>() / results.len() as f64
    };
    let avg_decode = if results.is_empty() {
        1.0
    } else {
        results.iter().map(|r| r.decode_speedup).sum::<f64>() / results.len() as f64
    };
    let total_encode_fs = if results.is_empty() {
        0.0
    } else {
        results.iter().map(|r| r.fastserial_encode_ms).sum::<f64>() / results.len() as f64
    };
    let total_encode_sj = if results.is_empty() {
        0.0
    } else {
        results.iter().map(|r| r.serde_json_encode_ms).sum::<f64>() / results.len() as f64
    };
    let total_decode_fs = if results.is_empty() {
        0.0
    } else {
        results.iter().map(|r| r.fastserial_decode_ms).sum::<f64>() / results.len() as f64
    };
    let total_decode_sj = if results.is_empty() {
        0.0
    } else {
        results.iter().map(|r| r.serde_json_decode_ms).sum::<f64>() / results.len() as f64
    };

    let test_names: Vec<String> = results.iter().map(|r| r.test_type.clone()).collect();
    let test_names_js = test_names
        .iter()
        .map(|n| format!("'{}'", n.replace("'", "\\'")))
        .collect::<Vec<_>>()
        .join(",");
    let fs_encode_data = results
        .iter()
        .map(|r| format!("{:.2}", r.fastserial_encode_ms))
        .collect::<Vec<_>>()
        .join(",");
    let sj_encode_data = results
        .iter()
        .map(|r| format!("{:.2}", r.serde_json_encode_ms))
        .collect::<Vec<_>>()
        .join(",");
    let fs_decode_data = results
        .iter()
        .map(|r| format!("{:.2}", r.fastserial_decode_ms))
        .collect::<Vec<_>>()
        .join(",");
    let sj_decode_data = results
        .iter()
        .map(|r| format!("{:.2}", r.serde_json_decode_ms))
        .collect::<Vec<_>>()
        .join(",");

    let mut table_rows = String::new();
    for r in results {
        table_rows.push_str(&format!(r#"<tr><td><strong>{}</strong></td><td>{}</td><td>{:.2} ms</td><td>{:.2} ms</td><td>{:.2} ms</td><td>{:.2} ms</td><td class="speedup {}">{:.2}x</td><td class="speedup {}">{:.2}x</td></tr>"#,
            r.test_type, r.total_records, r.fastserial_encode_ms, r.serde_json_encode_ms, r.fastserial_decode_ms, r.serde_json_decode_ms,
            if r.encode_speedup >= 1.0 { "positive" } else { "negative" }, r.encode_speedup,
            if r.decode_speedup >= 1.0 { "positive" } else { "negative" }, r.decode_speedup
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>FastSerial Benchmark - {}</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'Segoe UI', Arial, sans-serif; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; padding: 20px; }}
        .container {{ max-width: 1400px; margin: 0 auto; }}
        h1 {{ color: white; text-align: center; margin-bottom: 10px; }}
        .subtitle {{ color: #e0e0e0; text-align: center; margin-bottom: 30px; }}
        .back-link {{ display: inline-block; padding: 10px 20px; background: white; color: #667eea; text-decoration: none; border-radius: 8px; margin-bottom: 20px; font-weight: 600; }}
        .summary {{ background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%); padding: 20px; border-radius: 12px; margin-bottom: 20px; color: white; display: grid; grid-template-columns: repeat(6, 1fr); gap: 15px; }}
        .summary-card {{ background: rgba(255,255,255,0.2); padding: 15px; border-radius: 8px; text-align: center; }}
        .summary-card .value {{ font-size: 2em; font-weight: bold; }}
        .summary-card .label {{ font-size: 0.9em; opacity: 0.9; }}
        .card {{ background: white; padding: 20px; border-radius: 12px; margin-bottom: 20px; }}
        .card h2 {{ color: #333; margin-bottom: 15px; border-bottom: 2px solid #667eea; padding-bottom: 10px; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #eee; }}
        th {{ background: #667eea; color: white; }}
        .speedup {{ font-weight: bold; }}
        .speedup.positive {{ color: #11998e; }}
        .speedup.negative {{ color: #e74c3c; }}
        .chart-container {{ position: relative; height: 400px; margin: 20px 0; }}
        .legend {{ background: white; padding: 15px; border-radius: 8px; margin-bottom: 20px; display: flex; gap: 30px; justify-content: center; }}
        .legend span {{ display: flex; align-items: center; gap: 8px; }}
        .legend .dot {{ width: 16px; height: 16px; border-radius: 4px; }}
        .legend .dot.fs {{ background: #11998e; }}
        .legend .dot.sj {{ background: #667eea; }}
    </style>
</head>
<body>
    <div class="container">
        <a href="/" class="back-link">← Back</a>
        <h1>FastSerial Benchmark Report</h1>
        <p class="subtitle">Sample: {} | Type: {}</p>
        
        <div class="legend">
            <span><span class="dot fs"></span> <strong>fs</strong> = fastserial (our library)</span>
            <span><span class="dot sj"></span> <strong>sj</strong> = serde_json (comparison)</span>
        </div>
        
        <div class="summary">
            <div class="summary-card"><div class="value">{:.2}x</div><div class="label">Avg Encode</div></div>
            <div class="summary-card"><div class="value">{:.2}x</div><div class="label">Avg Decode</div></div>
            <div class="summary-card"><div class="value">{:.2} ms</div><div class="label">fastserial encode</div></div>
            <div class="summary-card"><div class="value">{:.2} ms</div><div class="label">serde_json encode</div></div>
            <div class="summary-card"><div class="value">{:.2} ms</div><div class="label">fastserial decode</div></div>
            <div class="summary-card"><div class="value">{:.2} ms</div><div class="label">serde_json decode</div></div>
        </div>
        
        <div class="card">
            <h2>Performance Chart</h2>
            <div class="chart-container"><canvas id="chart"></canvas></div>
        </div>
        
        <div class="card">
            <h2>Speedup</h2>
            <div class="chart-container"><canvas id="speedupChart"></canvas></div>
        </div>
        
        <div class="card">
            <h2>Results</h2>
            <table>
            <thead>
                <tr>
                    <th>Test Type</th>
                    <th>Records</th>
                    <th>fastserial encode (ms)</th>
                    <th>serde_json encode (ms)</th>
                    <th>fastserial decode (ms)</th>
                    <th>serde_json decode (ms)</th>
                    <th>Encode Speedup</th>
                    <th>Decode Speedup</th>
                </tr>
            </thead><tbody>{}</tbody></table>
        </div>
    </div>
    
    <script>
        const testNames = [{}];
        const fsEncodeData = [{}];
        const sjEncodeData = [{}];
        const fsDecodeData = [{}];
        const sjDecodeData = [{}];
        
        new Chart(document.getElementById('chart'), {{
            type: 'bar',
            data: {{
                labels: testNames,
                datasets: [
                    {{ label: 'fs encode', data: fsEncodeData, backgroundColor: '#11998e' }},
                    {{ label: 'sj encode', data: sjEncodeData, backgroundColor: '#667eea' }},
                    {{ label: 'fs decode', data: fsDecodeData, backgroundColor: '#38ef7d' }},
                    {{ label: 'sj decode', data: sjDecodeData, backgroundColor: '#f39c12' }}
                ]
            }},
            options: {{ responsive: true, plugins: {{ legend: {{ position: 'top' }} }}, scales: {{ y: {{ beginAtZero: true }} }} }}
        }});
        
        new Chart(document.getElementById('speedupChart'), {{
            type: 'bar',
            data: {{
                labels: testNames,
                datasets: [
                    {{ label: 'Encode Speedup', data: fsEncodeData.map((v,i)=>sjEncodeData[i]/v), backgroundColor: '#9b59b6' }},
                    {{ label: 'Decode Speedup', data: fsDecodeData.map((v,i)=>sjDecodeData[i]/v), backgroundColor: '#e74c3c' }}
                ]
            }},
            options: {{ responsive: true, plugins: {{ legend: {{ position: 'top' }} }}, scales: {{ y: {{ beginAtZero: true }} }} }}
        }});
    </script>
</body>
</html>"#,
        report_id,
        sample_size,
        json_type,
        avg_encode,
        avg_decode,
        total_encode_fs,
        total_encode_sj,
        total_decode_fs,
        total_decode_sj,
        table_rows,
        test_names_js,
        fs_encode_data,
        sj_encode_data,
        fs_decode_data,
        sj_decode_data
    )
}

async fn run_simple_user_benchmark(sample_size: i32) -> BatchReport {
    let json_data = fs::read_to_string("api-bench/jsondata/simple_user.json")
        .unwrap_or_else(|_| "{}".to_string());

    let start = Instant::now();
    for _ in 0..sample_size {
        let _: SimpleUser = serde_json::from_str(&json_data).unwrap();
    }
    let serde_decode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _: SimpleUser = fastserial::json::decode(json_data.as_bytes()).unwrap();
    }
    let fastserial_decode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let user = SimpleUser {
        id: 1,
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        age: 28,
        is_active: true,
    };

    let start = Instant::now();
    for _ in 0..sample_size {
        let _ = serde_json::to_vec(&user).unwrap();
    }
    let serde_encode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _ = fastserial::json::encode(&user).unwrap();
    }
    let fastserial_encode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let encode_speedup = if fastserial_encode_ms > 0.0 {
        serde_encode_ms / fastserial_encode_ms
    } else {
        1.0
    };
    let decode_speedup = if fastserial_decode_ms > 0.0 {
        serde_decode_ms / fastserial_decode_ms
    } else {
        1.0
    };

    BatchReport {
        test_type: "Simple User".to_string(),
        sample_size,
        total_records: sample_size,
        fastserial_encode_ms,
        serde_json_encode_ms: serde_encode_ms,
        fastserial_decode_ms,
        serde_json_decode_ms: serde_decode_ms,
        encode_speedup,
        decode_speedup,
    }
}

async fn run_batch_users_benchmark(sample_size: i32) -> BatchReport {
    let json_data = fs::read_to_string("api-bench/jsondata/batch_users.json")
        .unwrap_or_else(|_| "[]".to_string());
    let users: Vec<SimpleUser> = serde_json::from_str(&json_data).unwrap();
    let record_count = users.len() as i32;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _: Vec<SimpleUser> = serde_json::from_str(&json_data).unwrap();
    }
    let serde_decode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _: Vec<SimpleUser> = fastserial::json::decode(json_data.as_bytes()).unwrap();
    }
    let fastserial_decode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _ = serde_json::to_vec(&users).unwrap();
    }
    let serde_encode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _ = fastserial::json::encode(&users).unwrap();
    }
    let fastserial_encode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let encode_speedup = if fastserial_encode_ms > 0.0 {
        serde_encode_ms / fastserial_encode_ms
    } else {
        1.0
    };
    let decode_speedup = if fastserial_decode_ms > 0.0 {
        serde_decode_ms / fastserial_decode_ms
    } else {
        1.0
    };

    BatchReport {
        test_type: format!("Batch ({} users)", record_count),
        sample_size,
        total_records: record_count * sample_size,
        fastserial_encode_ms,
        serde_json_encode_ms: serde_encode_ms,
        fastserial_decode_ms,
        serde_json_decode_ms: serde_decode_ms,
        encode_speedup,
        decode_speedup,
    }
}

async fn run_product_benchmark(sample_size: i32) -> BatchReport {
    let json_data = fs::read_to_string("api-bench/jsondata/complex_product.json")
        .unwrap_or_else(|_| "{}".to_string());

    let start = Instant::now();
    for _ in 0..sample_size {
        let _: Product = serde_json::from_str(&json_data).unwrap();
    }
    let serde_decode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _: Product = fastserial::json::decode(json_data.as_bytes()).unwrap();
    }
    let fastserial_decode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let product = Product {
        id: 101,
        name: "Test".to_string(),
        description: "Desc".to_string(),
        price: 99.99,
        category: "Test".to_string(),
        tags: vec!["test".to_string()],
        stock: 10,
        is_available: true,
        weight_kg: 1.0,
        dimensions: crate::models::Dimensions {
            width_cm: 10.0,
            height_cm: 10.0,
            depth_cm: 10.0,
        },
        specs: crate::models::ProductSpecs {
            battery_life_hours: 10,
            connectivity: vec!["usb".to_string()],
            driver_size_mm: 10,
            impedance_ohm: 32,
            frequency_response: "20Hz".to_string(),
        },
        variants: vec![],
    };

    let start = Instant::now();
    for _ in 0..sample_size {
        let _ = serde_json::to_vec(&product).unwrap();
    }
    let serde_encode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _ = fastserial::json::encode(&product).unwrap();
    }
    let fastserial_encode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let encode_speedup = if fastserial_encode_ms > 0.0 {
        serde_encode_ms / fastserial_encode_ms
    } else {
        1.0
    };
    let decode_speedup = if fastserial_decode_ms > 0.0 {
        serde_decode_ms / fastserial_decode_ms
    } else {
        1.0
    };

    BatchReport {
        test_type: "Complex Product".to_string(),
        sample_size,
        total_records: sample_size,
        fastserial_encode_ms,
        serde_json_encode_ms: serde_encode_ms,
        fastserial_decode_ms,
        serde_json_decode_ms: serde_decode_ms,
        encode_speedup,
        decode_speedup,
    }
}

async fn run_order_benchmark(sample_size: i32) -> BatchReport {
    let json_data = fs::read_to_string("api-bench/jsondata/nested_order.json")
        .unwrap_or_else(|_| "{}".to_string());

    let start = Instant::now();
    for _ in 0..sample_size {
        let _: Order = serde_json::from_str(&json_data).unwrap();
    }
    let serde_decode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _: Order = fastserial::json::decode(json_data.as_bytes()).unwrap();
    }
    let fastserial_decode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let order = Order {
        order_id: "ORD-001".to_string(),
        customer: crate::models::Customer {
            id: 1,
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
            phone: "123".to_string(),
            address: crate::models::Address {
                street: "123 St".to_string(),
                city: "City".to_string(),
                state: "ST".to_string(),
                zip: "12345".to_string(),
                country: "USA".to_string(),
            },
        },
        items: vec![crate::models::OrderItem {
            product_id: 1,
            name: "Item".to_string(),
            quantity: 1,
            unit_price: 10.0,
            subtotal: 10.0,
        }],
        subtotal: 10.0,
        tax: 1.0,
        shipping: 1.0,
        total: 12.0,
        status: "pending".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };

    let start = Instant::now();
    for _ in 0..sample_size {
        let _ = serde_json::to_vec(&order).unwrap();
    }
    let serde_encode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _ = fastserial::json::encode(&order).unwrap();
    }
    let fastserial_encode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let encode_speedup = if fastserial_encode_ms > 0.0 {
        serde_encode_ms / fastserial_encode_ms
    } else {
        1.0
    };
    let decode_speedup = if fastserial_decode_ms > 0.0 {
        serde_decode_ms / fastserial_decode_ms
    } else {
        1.0
    };

    BatchReport {
        test_type: "Nested Order".to_string(),
        sample_size,
        total_records: sample_size,
        fastserial_encode_ms,
        serde_json_encode_ms: serde_encode_ms,
        fastserial_decode_ms,
        serde_json_decode_ms: serde_decode_ms,
        encode_speedup,
        decode_speedup,
    }
}

async fn run_blog_post_benchmark(sample_size: i32) -> BatchReport {
    let json_data = fs::read_to_string("api-bench/jsondata/blog_post.json")
        .unwrap_or_else(|_| "{}".to_string());

    let start = Instant::now();
    for _ in 0..sample_size {
        let _: BlogPost = serde_json::from_str(&json_data).unwrap();
    }
    let serde_decode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _: BlogPost = fastserial::json::decode(json_data.as_bytes()).unwrap();
    }
    let fastserial_decode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let blog_post = BlogPost {
        id: 1,
        title: "Test Post".to_string(),
        slug: "test".to_string(),
        content: "Content".to_string(),
        excerpt: "Excerpt".to_string(),
        author: crate::models::Author {
            id: 1,
            username: "author".to_string(),
            display_name: "Author".to_string(),
            bio: "Bio".to_string(),
            avatar_url: "https://example.com/avatar.jpg".to_string(),
            social_links: crate::models::SocialLinks {
                twitter: Some("@author".to_string()),
                github: Some("github.com/author".to_string()),
            },
        },
        category: crate::models::Category {
            id: 1,
            name: "Tech".to_string(),
            slug: "tech".to_string(),
        },
        tags: vec!["rust".to_string()],
        featured_image: "https://example.com/image.jpg".to_string(),
        status: "published".to_string(),
        view_count: 1000,
        like_count: 100,
        comment_count: 10,
        reading_time_minutes: 5,
        published_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        comments: vec![],
        related_posts: vec![],
    };

    let start = Instant::now();
    for _ in 0..sample_size {
        let _ = serde_json::to_vec(&blog_post).unwrap();
    }
    let serde_encode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    for _ in 0..sample_size {
        let _ = fastserial::json::encode(&blog_post).unwrap();
    }
    let fastserial_encode_ms = start.elapsed().as_secs_f64() * 1000.0;

    let encode_speedup = if fastserial_encode_ms > 0.0 {
        serde_encode_ms / fastserial_encode_ms
    } else {
        1.0
    };
    let decode_speedup = if fastserial_decode_ms > 0.0 {
        serde_decode_ms / fastserial_decode_ms
    } else {
        1.0
    };

    BatchReport {
        test_type: "Blog Post".to_string(),
        sample_size,
        total_records: sample_size,
        fastserial_encode_ms,
        serde_json_encode_ms: serde_encode_ms,
        fastserial_decode_ms,
        serde_json_decode_ms: serde_decode_ms,
        encode_speedup,
        decode_speedup,
    }
}
