use globe::{CameraConfig, Canvas, GlobeConfig, GlobeTemplate};
use gloo_net::http::Request;
use gloo_timers::callback::Interval;
use shared::Analytics;
use std::collections::HashMap;
use std::f64::consts::PI;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlCanvasElement, HtmlSelectElement};
use yew::html;
use yew::html::HtmlResult;
use yew::suspense::use_future;
use yew::Suspense;
use yew::{
    function_component, use_effect_with_deps, use_node_ref, use_state, Callback, Html, Properties,
};

fn format_count(count: &i32) -> Html {
    html! {
        <span class="count-badge">{ count }</span>
    }
}

fn to_html(analytics: &Analytics) -> Html {
    html! {
        <tr>
            <td class="ip-cell">{ &analytics.ip_address }</td>
            <td class="path-cell">
                <span class="path-prefix">{"/"}</span>
                { analytics.path.trim_start_matches('/') }
            </td>
            <td class="country-cell">
                <span class="country-code">{ &analytics.iso_code }</span>
            </td>
            <td class="count-cell">{ format_count(&(analytics.count as i32)) }</td>
        </tr>
    }
}

#[function_component(LoadingSpinner)]
fn loading_spinner() -> Html {
    html! {
        <div class="loading-container">
            <div class="loading-spinner"></div>
            <span class="loading-text">{"Loading analytics..."}</span>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ToolbarProps {
    on_filter_change: Callback<String>,
    on_view_change: Callback<ViewType>,
    view_type: ViewType,
}

#[derive(PartialEq, Clone)]
pub enum ViewType {
    Table,
    Charts,
}

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    data: Vec<Analytics>,
}

#[function_component(AnalyticsToolbar)]
fn analytics_toolbar(props: &ToolbarProps) -> Html {
    html! {
        <div class="tools-section">
            <div class="filter-group">
                <label>{"Country:"}</label>
                <select onchange={props.on_filter_change.reform(|e: Event| {
                    let select = e.target().unwrap().unchecked_into::<HtmlSelectElement>();
                    select.value()
                })}>
                    <option value="">{"All Countries"}</option>
                    <option value="US">{"United States"}</option>
                    <option value="GB">{"United Kingdom"}</option>
                    // Add more countries as needed
                </select>
            </div>
            <div class="view-toggle">
                <button
                    class={if props.view_type == ViewType::Table { "active" } else { "" }}
                    onclick={props.on_view_change.reform(|_| ViewType::Table)}>
                    {"Table View"}
                </button>
                <button
                    class={if props.view_type == ViewType::Charts { "active" } else { "" }}
                    onclick={props.on_view_change.reform(|_| ViewType::Charts)}>
                    {"Charts"}
                </button>
            </div>
        </div>
    }
}

#[function_component(VisitsByCountryChart)]
fn visits_by_country_chart(props: &ChartProps) -> Html {
    // Group data by country
    let mut country_visits: HashMap<String, i32> = HashMap::new();
    for analytics in props.data.iter() {
        *country_visits
            .entry(analytics.iso_code.clone())
            .or_insert(0) += analytics.count as i32;
    }

    let mut pairs: Vec<_> = country_visits.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));
    let pairs = pairs.into_iter().take(10).collect::<Vec<_>>();

    let max_value = *pairs.iter().map(|(_, v)| v).max().unwrap_or(&0);
    let bar_width = 40;
    let spacing = 20;
    let height = 300;
    let width = (bar_width + spacing) * pairs.len() as i32;

    html! {
        <svg width={width.to_string()} height={height.to_string()} class="analytics-chart">
            // Bars
            {
                pairs.iter().enumerate().map(|(i, (country, value))| {
                    let x = (i as i32 * (bar_width + spacing)) + spacing/2;
                    let bar_height = (*value as f32 / max_value as f32 * (height - 60) as f32) as i32;
                    let y = height - bar_height - 30;
                    let hue = (i as f32 / pairs.len() as f32 * 360.0) as i32;

                    html! {
                        <>
                            <rect
                                x={x.to_string()}
                                y={y.to_string()}
                                width={bar_width.to_string()}
                                height={bar_height.to_string()}
                                fill={format!("hsl({}, 80%, 60%)", hue)}
                            />
                            <text
                                x={(x + bar_width/2).to_string()}
                                y={(height - 10).to_string()}
                                text-anchor="middle"
                                class="analytics-chart-label"
                            >
                                {country}
                            </text>
                            <text
                                x={(x + bar_width/2).to_string()}
                                y={(y - 5).to_string()}
                                text-anchor="middle"
                                class="analytics-chart-value"
                            >
                                {value}
                            </text>
                        </>
                    }
                }).collect::<Html>()
            }
        </svg>
    }
}

#[function_component(TopPagesChart)]
fn top_pages_chart(props: &ChartProps) -> Html {
    // Group data by page
    let mut page_visits: HashMap<String, i32> = HashMap::new();
    for analytics in props.data.iter() {
        *page_visits.entry(analytics.path.clone()).or_insert(0) += analytics.count as i32;
    }

    let mut pairs: Vec<_> = page_visits.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));
    let pairs = pairs.into_iter().take(5).collect::<Vec<_>>();

    html! {
        <div class="analytics-pages-list">
            { pairs.iter().enumerate().map(|(i, (path, count))| {
                let hue = (i as f32 / pairs.len() as f32 * 360.0) as i32;
                html! {
                    <div class="analytics-page-item">
                        <div class="analytics-page-rank" style={format!("background: hsl({}, 80%, 60%)", hue)}>
                            {format!("#{}", i + 1)}
                        </div>
                        <div class="analytics-page-info">
                            <div class="analytics-page-path">{path}</div>
                            <div class="analytics-page-visits">{format!("{} visits", count)}</div>
                        </div>
                    </div>
                }
            }).collect::<Html>() }
        </div>
    }
}

#[function_component(VisitsOverTimeChart)]
fn visits_over_time_chart(props: &ChartProps) -> Html {
    let total_visits: Vec<(usize, i32)> = props
        .data
        .iter()
        .enumerate()
        .map(|(i, a)| (i, a.count as i32))
        .collect();

    if total_visits.is_empty() {
        return html! { <div class="no-data">{"No data available"}</div> };
    }

    let max_value = total_visits.iter().map(|(_, v)| *v).max().unwrap_or(0);
    let width = 500i32;
    let height = 300i32;
    let padding = 40i32;

    let points: String = total_visits
        .iter()
        .enumerate()
        .map(|(i, (_, value))| {
            let x = padding as f32
                + (i as f32 / (total_visits.len() - 1) as f32 * (width - 2 * padding) as f32);
            let y = height as f32
                - padding as f32
                - (*value as f32 / max_value as f32 * (height - 2 * padding) as f32);
            if i == 0 {
                format!("M {},{}", x, y)
            } else {
                format!("L {},{}", x, y)
            }
        })
        .collect();

    html! {
        <svg width={width.to_string()} height={height.to_string()} class="analytics-chart">
            // Grid lines
            {
                (0..=4).map(|i| {
                    let y = height as f32 - padding as f32 - (i as f32 * (height - 2 * padding) as f32 / 4.0);
                    let value = (i as f32 * max_value as f32 / 4.0) as i32;
                    html! {
                        <>
                            <line
                                x1={padding.to_string()}
                                y1={y.to_string()}
                                x2={(width - padding).to_string()}
                                y2={y.to_string()}
                                class="analytics-grid-line"
                            />
                            <text
                                x={(padding - 5).to_string()}
                                y={y.to_string()}
                                text-anchor="end"
                                alignment-baseline="middle"
                                class="analytics-chart-label"
                            >
                                {value}
                            </text>
                        </>
                    }
                }).collect::<Html>()
            }

            // Line
            <path
                d={points}
                class="analytics-trend-line"
                fill="none"
                stroke="#ff458a"
                stroke-width="2"
            />

            // Points
            {
                total_visits.iter().enumerate().map(|(i, (_, value))| {
                    let x = padding as f32 + (i as f32 / (total_visits.len() - 1) as f32 * (width - 2 * padding) as f32);
                    let y = height as f32 - padding as f32 - (*value as f32 / max_value as f32 * (height - 2 * padding) as f32);
                    html! {
                        <circle
                            cx={x.to_string()}
                            cy={y.to_string()}
                            r="4"
                            class="analytics-data-point"
                        />
                    }
                }).collect::<Html>()
            }
        </svg>
    }
}

#[function_component(AnalyticsCharts)]
fn analytics_charts(props: &ChartProps) -> Html {
    html! {
        <div class="analytics-charts">
            <div class="analytics-charts-section">
                <div class="analytics-chart-container">
                    <h3 class="analytics-chart-title">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
                        </svg>
                        {"Visits by Country"}
                    </h3>
                    <VisitsByCountryChart data={props.data.clone()} />
                </div>
                <div class="analytics-chart-container">
                    <h3 class="analytics-chart-title">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                            <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z"/>
                        </svg>
                        {"Top Pages"}
                    </h3>
                    <TopPagesChart data={props.data.clone()} />
                </div>
                <div class="analytics-chart-container">
                    <h3 class="analytics-chart-title">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                            <path d="M3.5 18.49l6-6.01 4 4L22 6.92l-1.41-1.41-7.09 7.97-4-4L2 16.99z"/>
                        </svg>
                        {"Visits Over Time"}
                    </h3>
                    <VisitsOverTimeChart data={props.data.clone()} />
                </div>
            </div>
        </div>
    }
}

#[function_component(AnalyticsHeader)]
fn analytics_header() -> Html {
    html! {
        <div class="header">
            <div class="title-section">
                <h1>{"Analytics Dashboard"}</h1>
                <p class="subtitle">{"Real-time visitor insights"}</p>
            </div>
        </div>
    }
}

#[function_component(AsciiGlobe)]
fn ascii_globe(props: &ChartProps) -> Html {
    let rotation = use_state(|| 0.0f64);
    let globe_ref = use_node_ref();

    // Set up globe rotation animation
    {
        let rotation = rotation.clone();
        use_effect_with_deps(
            move |_| {
                let interval = Interval::new(50, move || {
                    rotation.set(*rotation + 0.02);
                });
                || {
                    interval.forget();
                }
            },
            (),
        );
    }

    // Update globe rendering
    {
        let globe_ref = globe_ref.clone();
        let rotation_val = *rotation;

        use_effect_with_deps(
            move |_| {
                if let Some(pre) = globe_ref.cast::<web_sys::Element>() {
                    // Create and configure globe
                    let mut globe = GlobeConfig::new()
                        .use_template(GlobeTemplate::Earth)
                        .with_camera(CameraConfig::default())
                        .build();

                    // Create canvas and render globe
                    let mut canvas = Canvas::new(80, 40, None);
                    globe.render_on(&mut canvas);

                    // Convert to HTML string
                    let mut globe_str = String::new();
                    let (size_x, size_y) = canvas.get_size();

                    for i in 0..size_y / 8 {
                        for j in 0..size_x / 4 {
                            globe_str.push_str(&canvas.matrix[i][j].to_string());
                        }
                        globe_str.push('\n');
                    }

                    // Update the pre element
                    pre.set_inner_html(&globe_str);
                }
                || ()
            },
            rotation_val,
        );
    }

    // Visitor dots overlay
    let country_positions = [
        ("US", (8.0, 4.0)),  // United States
        ("GB", (13.0, 3.0)), // United Kingdom
        ("DE", (14.0, 3.0)), // Germany
        ("FR", (13.0, 4.0)), // France
        ("ES", (12.0, 4.0)), // Spain
        ("IT", (14.0, 4.0)), // Italy
        ("JP", (20.0, 4.0)), // Japan
        ("CN", (18.0, 4.0)), // China
        ("IN", (17.0, 5.0)), // India
        ("AU", (19.0, 8.0)), // Australia
        ("BR", (11.0, 7.0)), // Brazil
        ("CA", (8.0, 3.0)),  // Canada
        ("RU", (16.0, 3.0)), // Russia
        ("ZA", (14.0, 8.0)), // South Africa
        ("MX", (7.0, 5.0)),  // Mexico
        ("AR", (10.0, 8.0)), // Argentina
    ];

    let visitor_dots = props
        .data
        .iter()
        .map(|analytics| {
            if let Some(&(_, (mut x, y))) = country_positions
                .iter()
                .find(|(code, _)| *code == analytics.iso_code.as_str())
            {
                // Adjust x position based on rotation
                x = (x + *rotation * 10.0) % 24.0;
                if x < 4.0 || x > 20.0 {
                    return html! {}; // Don't show dots on the "back" of the globe
                }

                let background = if analytics.count > 10 {
                    "var(--chart-accent)"
                } else {
                    "#7b5fff"
                };

                let style = format!(
                    "left: {}em; top: {}em; background: {}; opacity: {};",
                    x,
                    y,
                    background,
                    // Fade dots near the edges
                    if x < 6.0 || x > 18.0 { 0.5 } else { 1.0 }
                );

                html! {
                    <div class="visitor-dot" {style}>
                        <div class="visitor-pulse"></div>
                    </div>
                }
            } else {
                html! {}
            }
        })
        .collect::<Html>();

    html! {
        <div class="ascii-globe">
            <div class="globe-container">
                <pre ref={globe_ref} class="globe-text"></pre>
                { visitor_dots }
            </div>
            <div class="globe-legend">
                <div class="legend-item active">
                    <span class="dot"></span>
                    <span>{"Active Visitors"}</span>
                </div>
                <div class="legend-item recent">
                    <span class="dot"></span>
                    <span>{"Recent Visitors"}</span>
                </div>
            </div>
        </div>
    }
}

#[function_component(AnalyticsContent)]
fn analytics_content() -> HtmlResult {
    let view_type = use_state(|| ViewType::Table);
    let country_filter = use_state(String::new);

    let resp = use_future(|| async {
        Request::get("/api/analytics")
            .send()
            .await?
            .json::<Vec<Analytics>>()
            .await
    })?;

    match *resp {
        Ok(ref res) => {
            let filtered_data = res
                .iter()
                .filter(|a| country_filter.is_empty() || &a.iso_code == &*country_filter)
                .cloned()
                .collect::<Vec<Analytics>>();

            let analytics_table = filtered_data.iter().map(|a| to_html(a)).collect::<Html>();

            Ok(html! {
                <div class="analytics">
                    <AnalyticsHeader />
                    <AsciiGlobe data={filtered_data.clone()} />
                    <AnalyticsToolbar
                        on_filter_change={Callback::from(move |c| country_filter.set(c))}
                        on_view_change={
                            let view_type = view_type.clone();
                            Callback::from(move |v| view_type.set(v))
                        }
                        view_type={(*view_type).clone()}
                    />
                    {
                        if *view_type == ViewType::Table {
                            html! {
                                <div class="table-container">
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>{"IP Address"}</th>
                                                <th>{"Path"}</th>
                                                <th>{"Country"}</th>
                                                <th>{"Visits"}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            { analytics_table }
                                        </tbody>
                                    </table>
                                </div>
                            }
                        } else {
                            html! { <AnalyticsCharts data={filtered_data} /> }
                        }
                    }
                </div>
            })
        }
        Err(ref failure) => Ok(html! {
            <div class="error-container">
                <div class="error-message">
                    <span class="error-icon">{"⚠️"}</span>
                    <p>{"Failed to load analytics data"}</p>
                    <p class="error-details">{ failure.to_string() }</p>
                </div>
            </div>
        }),
    }
}

#[function_component(AnalyticsComponent)]
pub fn analytics() -> Html {
    html! {
        <Suspense fallback={html! { <LoadingSpinner /> }}>
            <AnalyticsContent />
        </Suspense>
    }
}
