from datetime import datetime, timedelta
import logging
import time
from uuid import uuid4
from flask import Flask, jsonify, request, redirect, make_response
import jwt

JWT_TOKEN_EXPIRE_TIME = 3600 * 2  # token有效时间 2小时
JWT_SECRET = "sso-3E0C07FFFCFFF3E00E0039FCE00E7F387"  # 加解密密钥
JWT_ALGORITHM = "HS256"  # 加解密算法
# 设置日志
logging.basicConfig(
    level=logging.INFO,
    filename="sso.log",
    format="%(asctime)s - %(levelname)s - %(message)s",
)

app = Flask(__name__)
app.secret_key = "idp-7F39F01FCE7FFE0318001C0670070E03F"


tokens = []
# 存储用户名和密码（示例用，实际应用中应使用数据库）
users = {"user1": "123", "user2": "123"}


def generate_jwt_token(username: str) -> str:
    """根据用户id生成token"""
    payload = {"username": username, "exp": int(time.time()) + JWT_TOKEN_EXPIRE_TIME}
    token = jwt.encode(payload, JWT_SECRET, algorithm=JWT_ALGORITHM)
    return token


def verify_jwt_token(username: str, token: str) -> bool:
    """验证用户token"""
    payload = {"username": username}
    try:
        _payload = jwt.decode(token, JWT_SECRET, algorithms=[JWT_ALGORITHM])
    except jwt.PyJWTError:
        logging.info("token verification failed")
        return False
    else:
        logging.info(_payload)
        exp = int(_payload.pop("exp"))
        if time.time() > exp:
            logging.info("token expired")
            return False
        return payload == _payload


@app.route("/login", methods=["GET", "POST"])
def login():
    # 如果是POST请求，尝试执行登录逻辑
    if request.method == "POST":
        username = request.form.get("username")
        password = request.form.get("password")
        redirect_url = request.args.get("redirect_url", "")

        if users.get(username) == password:  # 简单的认证逻辑
            logging.info(f"User {username} logged in.")

            token = generate_jwt_token(username)
            tokens.append(token)
            # 登录成功后，重定向到之前请求的服务
            response = make_response(redirect(redirect_url))
            response.set_cookie(
                "auth_token", token, httponly=True, secure=True
            )  # 在生产环境中使用secure=True
            # 生成响应
            # response = jsonify({'message': 'You are logged in successfully.', 'token': token})
            return response
        else:
            logging.warning(f"Failed login attempt for username: {username}")
            return "Login Failed! Invalid username or password.", 401

    # 如果是GET请求，展示登录表单
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
    if token in tokens:
        del tokens[token]
        logging.info(f"Token {token} invalidated for logout.")
        return "Logged out successfully.", 200
    else:
        logging.warning(f"Attempt to logout with invalid or expired token: {token}")
        return "Invalid Token.", 403


@app.route("/verify", methods=["POST"])
def verify():
    # token = request.args.get('token', '')
    token = request.headers.get("Authorization", None)

    # token_info = tokens.get(token)
    try:
        payload = jwt.decode(token, JWT_SECRET, algorithms=[JWT_ALGORITHM])
    except jwt.PyJWTError:
        logging.info("token verification failed")
        return jsonify({"status": "invalid"}), 403

    # if not token_info:
    #     logging.warning(f"Invalid token verification attempt: {token}")
    #     return jsonify({"status": "invalid"}), 403

    if time.time() > payload["exp"]:
        # Token过期
        tokens.remove(token)  # 清理过期Token
        logging.warning(f"Expired token: {token}")
        return jsonify({"status": "expired"}), 403

    # Token有效
    logging.info(
        f"Token {token} verified successfully for username: {payload['username']}"
    )
    return jsonify({"status": "valid", "username": payload["username"]})


@app.route("/register", methods=["POST"])
def register():
    username = request.form.get("username")
    password = request.form.get("password")

    if username in users:
        logging.warning(f"Attempt to register with an existing username: {username}")
        return "Username already exists.", 400

    users[username] = password
    logging.info(f"New user registered: {username}")
    return "User registered successfully.", 201


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8000)
