# Sample Axum API with FastSerial

A high-performance RESTful API built with **Axum**, **SQLx (MySQL)**, and the custom **FastSerial** serialization library. This project demonstrates best practices for building scalable Rust web applications with JWT authentication and structured API responses.

## 🚀 Features

- **High-Performance Serialization**: Uses `fastserial` for lightning-fast JSON encoding/decoding.
- **Robust API Structure**: Standardized `ApiResponse<T>` for all endpoints.
- **JWT Authentication**: Secure endpoints with JSON Web Tokens.
- **Database Integration**: MySQL with SQLx, featuring automated schema initialization and data seeding.
- **Modern Error Handling**: Centralized error management with custom `AppError`.
- **Clean Architecture**: Separated handlers, models, routes, and database logic.

## 🛠 Prerequisites

- **Rust**: Latest stable version (1.80+ recommended).
- **MySQL**: A running instance (e.g., via Laragon or Docker).
- **Cargo**: Rust's package manager.

## ⚙️ Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/j4flmao/fastserial-rs.git
   cd fasterial-rs/sample-axum
   ```

2. **Configure Environment Variables**:
   Create a `.env` file in the `sample-axum` directory:
   ```env
   DATABASE_URL=mysql://user:password@localhost:3306/your_db_name
   JWT_SECRET=your_super_secret_key_change_me
   PORT=8082
   RUST_LOG=info,sample_axum=debug,tower_http=debug
   ```

3. **Database Initialization**:
   The application will automatically create the necessary tables (`users`, `categories`, `posts`) and seed initial data on the first run.

## 🏃 Running the Application

Before running, ensure your Cargo path is correctly set (especially on Windows):

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo run
```

The server will start at `http://127.0.0.1:8082`.

## 📖 API Endpoints

### Authentication & Users
- `POST /api/users/register`: Create a new account.
- `POST /api/users/login`: Authenticate and receive a JWT.
- `GET /api/users/profile`: Get current user profile (Requires JWT).

### Categories
- `GET /api/categories/`: List all categories.
- `POST /api/categories/`: Create a new category (Requires JWT).

### Posts
- `GET /api/posts/`: List all blog posts.
- `POST /api/posts/`: Create a new post (Requires JWT).

## 🧪 Seeding Data

On the first run, the system seeds:
- **Default Categories**: Technology, Rust, Web Development, Microservices.
- **Admin User**: 
    - Email: `admin@fastserial.rs`
    - Password: `admin123`
- **Welcome Post**: A sample post to get you started.

## 🛠 Development

### Performance Benchmarking
You can use `oha` to benchmark the API performance:
```powershell
oha -n 10000 -c 100 http://127.0.0.1:8082/api/posts/
```

### Code Style
Ensure your code matches the project standards:
```bash
cargo clippy
cargo fmt
```

## 📄 License
This project is part of the `fastserial-rs` workspace and is for experimental/demonstration purposes.
