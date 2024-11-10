source ./ip.sh

> "$HOME/Universitat/7.Semester/Bachelor/data_analysis/data/bw.log"
> "$HOME/Universitat/7.Semester/Bachelor/data_analysis/data/iops.log"
> "$HOME/Universitat/7.Semester/Bachelor/data_analysis/data/clat.log"
> "$HOME/Universitat/7.Semester/Bachelor/data_analysis/data/lat.log"

scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/bw_bw.1.log "$HOME/Universitat/7.Semester/Bachelor/data_analysis/data/bw.log"
scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/iops_iops.1.log "$HOME/Universitat/7.Semester/Bachelor/data_analysis/data/iops.log"
scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/lat_clat.1.log "$HOME/Universitat/7.Semester/Bachelor/data_analysis/data/clat.log"
scp -i ~/.ssh/mvp-key-pair.pem ec2-user@$IP:/home/ec2-user/lat_lat.1.log "$HOME/Universitat/7.Semester/Bachelor/data_analysis/data/lat.log"
