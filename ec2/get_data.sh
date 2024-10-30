source ./ip.sh

> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/bw.log"
> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/iops.log"
> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/clat.log"
> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/lat.log"

mkdir -p /tmp/remote_logs

rm -f /tmp/remote_logs/*

for i in {1..1}; do
  scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/bw_bw.$i.log /tmp/remote_logs/bw.$i.log
  cat /tmp/remote_logs/bw.$i.log >> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/bw.log"

  scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/iops_iops.$i.log /tmp/remote_logs/iops.$i.log
  cat /tmp/remote_logs/iops.$i.log >> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/iops.log"

  scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/lat_clat.$i.log /tmp/remote_logs/clat.$i.log
  cat /tmp/remote_logs/clat.$i.log >> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/clat.log"

  scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/lat_lat.$i.log /tmp/remote_logs/lat.$i.log
  cat /tmp/remote_logs/lat.$i.log >> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/lat.log"

done
