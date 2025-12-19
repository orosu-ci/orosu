# Getting Started

```bash
curl -fsSL https://orosu.dev/deb/public.key | sudo gpg --dearmor -o /usr/share/keyrings/orosu.gpg
echo "deb [signed-by=/usr/share/keyrings/orosu.gpg] https://orosu.dev/deb stable main" | sudo tee /etc/apt/sources.list.d/orosu.list
sudo apt update
sudo apt install orosu
```
