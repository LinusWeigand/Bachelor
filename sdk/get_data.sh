source ./ip.sh

> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/bw.log"
> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/iops.log"
> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/clat.log"
> "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/lat.log"

scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/bw_bw.1.log "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/bw.log"
scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/iops_iops.1.log "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/iops.log"
scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/lat_clat.1.log "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/clat.log"
scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/lat_lat.1.log "$HOME/Universität/7.Semester/Bachelor/data_analysis/data/latency/lat.log"
