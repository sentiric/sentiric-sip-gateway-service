```bash
sudo apt-get update && sudo apt-get install tcpdump -y
```

```bash
sudo tcpdump -i any -n udp port 5060 -A
```