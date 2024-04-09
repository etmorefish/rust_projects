from datetime import datetime, timedelta
import logging
from uuid import uuid4
from flask import Flask, jsonify, request, redirect, make_response
import jwt

# 设置日志
logging.basicConfig(level=logging.INFO, filename='sso.log',
                    format='%(asctime)s - %(levelname)s - %(message)s')

app = Flask(__name__)
app.secret_key = 'idp-7F39F01FCE7FFE0318001C0670070E03F'

sso_secret_key = 'sso-3E0C07FFFCFFF3E00E0039FCE00E7F387'

TOKEN_EXPIRATION_MINUTES = 30

tokens = {}
# 存储用户名和密码（示例用，实际应用中应使用数据库）
users = {"user1": "123","user2": "123"}


@app.route("/login", methods=["GET", "POST"])
def login():
    # 如果是POST请求，尝试执行登录逻辑
    if request.method == "POST":
        username = request.form.get("username")
        password = request.form.get("password")
        redirect_url = request.args.get("redirect_url", "")

        if users.get(username) == password:  # 简单的认证逻辑
            logging.info(f"User {username} logged in.")
            
            token = str(uuid4())  # 实际应用中应生成唯一Token
            # token = jwt.encode({'username': username}, sso_secret_key, algorithm='HS256')
            
            token_issue_time = datetime.now()
            tokens[token] = {"username": username, "issue_time": token_issue_time}
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

@app.route('/logout', methods=['GET'])
def logout():
    token = request.headers.get("Authorization", None)
    if token in tokens:
        del tokens[token]
        logging.info(f"Token {token} invalidated for logout.")
        return "Logged out successfully.", 200
    else:
        logging.warning(f"Attempt to logout with invalid or expired token: {token}")
        return "Invalid Token.", 403
    

@app.route("/verify", methods=["GET"])
def verify():
    # token = request.args.get('token', '')
    token = request.headers.get("Authorization", None)
    
    # decoded_token = jwt.decode(token, sso_secret_key, algorithms=['HS256'])
    
    token_info = tokens.get(token)

    if not token_info:
        logging.warning(f"Invalid token verification attempt: {token}")
        return jsonify({"status": "invalid"}), 403

    if datetime.now() - token_info["issue_time"] > timedelta(minutes=TOKEN_EXPIRATION_MINUTES):
        # Token过期
        del tokens[token]  # 清理过期Token
        logging.warning(f"Expired token: {token}")
        return jsonify({"status": "expired"}), 403

    # Token有效
    logging.info(f"Token {token} verified successfully for username: {token_info['username']}")
    return jsonify({"status": "valid", "username": token_info["username"]})

@app.route('/register', methods=['POST'])
def register():
    username = request.form.get('username')
    password = request.form.get('password')
    
    if username in users:
        logging.warning(f"Attempt to register with an existing username: {username}")
        return "Username already exists.", 400
    
    users[username] = password
    logging.info(f"New user registered: {username}")
    return "User registered successfully.", 201

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8000)
