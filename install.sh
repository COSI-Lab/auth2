cd target/debug
cp libnss_cosiauthd.so libnss_cosiauthd.so.2
sudo install -m 0644 libnss_cosiauthd.so.2 /lib
sudo /sbin/ldconfig -n /lib /usr/lib

