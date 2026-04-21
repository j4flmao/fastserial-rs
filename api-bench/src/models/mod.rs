use fastserial::{Decode, Encode};

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimpleUser {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub age: i32,
    pub is_active: bool,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dimensions {
    pub width_cm: f64,
    pub height_cm: f64,
    pub depth_cm: f64,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductSpecs {
    pub battery_life_hours: i32,
    pub connectivity: Vec<String>,
    pub driver_size_mm: i32,
    pub impedance_ohm: i32,
    pub frequency_response: String,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductVariant {
    pub color: String,
    pub sku: String,
    pub stock: i32,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub tags: Vec<String>,
    pub stock: i32,
    pub is_available: bool,
    pub weight_kg: f64,
    pub dimensions: Dimensions,
    pub specs: ProductSpecs,
    pub variants: Vec<ProductVariant>,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct Customer {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub address: Address,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderItem {
    pub product_id: i64,
    pub name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub subtotal: f64,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct Order {
    pub order_id: String,
    pub customer: Customer,
    pub items: Vec<OrderItem>,
    pub subtotal: f64,
    pub tax: f64,
    pub shipping: f64,
    pub total: f64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialLinks {
    pub twitter: Option<String>,
    pub github: Option<String>,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct Author {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: String,
    pub social_links: SocialLinks,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommentAuthor {
    pub id: i64,
    pub username: String,
    pub display_name: String,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommentReply {
    pub id: i64,
    pub author: CommentAuthor,
    pub content: String,
    pub likes: i32,
    pub created_at: String,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct Comment {
    pub id: i64,
    pub author: CommentAuthor,
    pub content: String,
    pub likes: i32,
    pub created_at: String,
    #[serde(default)]
    #[fastserial(default)]
    pub replies: Option<Vec<CommentReply>>,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelatedPost {
    pub id: i64,
    pub title: String,
    pub slug: String,
}

#[derive(Debug, Encode, Decode, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlogPost {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: String,
    pub author: Author,
    pub category: Category,
    pub tags: Vec<String>,
    pub featured_image: String,
    pub status: String,
    pub view_count: i64,
    pub like_count: i32,
    pub comment_count: i32,
    pub reading_time_minutes: i32,
    pub published_at: String,
    pub updated_at: String,
    pub comments: Vec<Comment>,
    pub related_posts: Vec<RelatedPost>,
}

#[derive(Debug, Encode, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchReport {
    pub test_type: String,
    pub sample_size: i32,
    pub total_records: i32,
    pub fastserial_encode_ms: f64,
    pub serde_json_encode_ms: f64,
    pub fastserial_decode_ms: f64,
    pub serde_json_decode_ms: f64,
    pub encode_speedup: f64,
    pub decode_speedup: f64,
}
