from datetime import datetime, timedelta
import os
from dotenv import load_dotenv
import logging
from logging.handlers import RotatingFileHandler
import time
from uuid import uuid4
from flask import Flask, jsonify, request, redirect, make_response
import jwt
from werkzeug.security import generate_password_hash, check_password_hash
from flask_talisman import Talisman

# 加载环境变量
load_dotenv()

# 应用配置
JWT_TOKEN_EXPIRE_TIME = 3600 * 2  # 2小时
JWT_SECRET = os.getenv("JWT_SECRET", "default_secret_key")  # 从环境变量读取
JWT_ALGORITHM = "HS256"
app = Flask(__name__)
app.secret_key = os.getenv("FLASK_SECRET_KEY", "default_flask_secret_key")
Talisman(app)  # 增加安全头

# 初始化日志记录
logging.basicConfig(level=logging.INFO)
handler = RotatingFileHandler('id.log', maxBytes=10000, backupCount=3)
logger = logging.getLogger(__name__)
logger.addHandler(handler)

tokens = {}
# 存储用户名和密码的示例（实际应用中应使用数据库）
users = {"user1": generate_password_hash("123"), "user2": generate_password_hash("123")}

def generate_jwt_token(username: str) -> str:
    """生成JWT token并存储元数据"""
    jti = str(uuid4())
    payload = {
        "username": username,
        "jti": jti,
        "exp": int(time.time()) + JWT_TOKEN_EXPIRE_TIME
    }
    token = jwt.encode(payload, JWT_SECRET, algorithm=JWT_ALGORITHM)
    tokens[jti] = {"username": username, "exp": payload["exp"]}
    return token

@app.route("/login", methods=["GET", "POST"])
def login():
    if request.method == "POST":
        username = request.form.get("username")
        password = request.form.get("password")
        redirect_url = request.args.get("redirect_url", "")

        if username in users and check_password_hash(users.get(username), password):
            logger.info(f"User {username} logged in.")
            token = generate_jwt_token(username)
            response = make_response(redirect(redirect_url))
            response.set_cookie("auth_token", token, httponly=True, secure=True)
            return response
        else:
            logger.warning(f"Failed login attempt for username: {username}")
            return "Login Failed! Invalid username or password.", 401
    return """
        <form method="post">
            Username: <input type="text" name="username"><br>
            Password: <input type="password" name="password"><br>
            <input type="submit" value="Login">
        </form>
    """

@app.route("/logout", methods=["GET"])
def logout():
    token = request.headers.get("Authorization", None)
    if not token:
        return "Token is missing.", 403

    try:
        payload = jwt.decode(token, JWT_SECRET, algorithms=[JWT_ALGORITHM])
    except jwt.PyJWTError:
        return "Invalid Token.", 403

    jti = payload.get("jti")
    if jti in tokens:
        del tokens[jti]
        return "Logged out successfully.", 200
    else:
        return "Invalid Token.", 403

@app.route("/verify", methods=["POST"])
def verify():
    token = request.headers.get("Authorization", None)
    if not token:
        logger.info("No token provided for verification.")
        return jsonify({"status": "invalid", "message": "No token provided"}), 403

    try:
        payload = jwt.decode(token, JWT_SECRET, algorithms=[JWT_ALGORITHM])
        jti = payload["jti"]
        if jti not in tokens or time.time() > tokens[jti]["exp"]:
            logger.info("Token is invalid or expired.")
            return jsonify({"status": "invalid", "message": "Token is invalid or expired"}), 403
    except jwt.ExpiredSignatureError:
        logger.info("Token has expired.")
        return jsonify({"status": "expired", "message": "Token has expired"}), 403
    except jwt.InvalidTokenError:
        logger.info("Token is invalid.")
        return jsonify({"status": "invalid", "message": "Token is invalid"}), 403

    logger.info(f"Token verified successfully for username: {payload['username']}")
    return jsonify({"status": "valid", "username": payload["username"]})

@app.route("/register", methods=["POST"])
def register():
    username = request.form.get("username")
    password = request.form.get("password")
    if username in users:
        logger.warning(f"Attempt to register with an existing username: {username}")
        return "Username already exists.", 400
    users[username] = generate_password_hash(password)
    logger.info(f"New user registered: {username}")
    return "User registered successfully.", 201

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8000)
