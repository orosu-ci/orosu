# Getting Started


```bash
curl -fsSL https://packages.nerdy.pro/NerdyPro.gpg | sudo gpg --dearmor -o /usr/share/keyrings/nerdy-pro.gpg
echo "deb [signed-by=/usr/share/keyrings/nerdy-pro.gpg] https://packages.nerdy.pro/ stable main" | sudo tee /etc/apt/sources.list.d/nerdy-pro.list
sudo apt update
sudo apt install orosu
```
