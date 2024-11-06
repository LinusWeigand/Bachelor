# Steps required to execute this program

## 1. Aws sdk needs access to your account:

- Create user in the aws console
- execute: `aws configure` in your console and type in your credentials

## 2. Add SSM Role

-- Create Role: AWS-Service, EC2, AmazonSSMManagedInstanceCore,
with name: EC2SSMRole

# Benchmarking Throughput & Latency

1. Install mdadm:
   sudo yum install mdadm

2. Create RAID array
   sudo mdadm --create /dev/md0 --level=0 --raid-devices=2 /dev/xvdb /dev/xvdc

3. Verify RAID array
   cat /proc/mdstat

4. Install xfsprogs
   sudo yum install xfsprogs

5. Format RAID array with XFS Filesystem
   sudo mkfs.xfs /dev/md0

6. Mount XFS Filesystem
   sudo mkdir /mnt/raid0
   sudo mount /dev/md0 /mnt/raid0

7. Make mount persistent
   echo '/dev/md0 /mnt/raid0 xfs defaults,noatime 0 0' | sudo tee -a /etc/fstab

8. Install fio
   sudo yum install fio

9. Run benchmark
   sudo fio --name=randrw --filename=/mnt/raid0/testfile --size=1G --bs=4k --rw=randrw --rwmixread=70 --numjobs=4 --runtime=60 --group_reporting

10. Run latency benchmark
    sudo fio --name=latency-test --filename=/mnt/raid0/latencyfile --size=1G --bs=4k --rw=randread --numjobs=4 --runtime=60 --group_reporting --latency-target=100 --latency-window=10

11. Sequential Write
    sudo fio --name=throughtput-test --filename=/mnt/raid0/testfile --rw=write --direct=1 --write_bw_log=bw --write_lat_log=lat --write_hist_log=hist --write_iops_log=iops --write_iolog=io --size=1G --bs=1M --numjobs=1 --iodepth=32 --runtime=180 --group_reporting --log_avg_msec=100 --time_based

12. Random Write
    sudo fio --name=throughtput-test --filename=/mnt/raid0/testfile --rw=randwrite --write_bw_log=bw --write_lat_log=lat --write_hist_log=hist --write_iops_log=iops --write_iolog=io --size=10G --bs=1M --numjobs=1 --iodepth=32 --runtime=180 --group_reporting --log_avg_msec=100 --time_based

13. Random Read
    sudo fio --name=throughtput-test --filename=/mnt/raid0/testfile --rw=randread --write_bw_log=bw --write_lat_log=lat --write_hist_log=hist --write_iops_log=iops --write_iolog=io --size=100G --bs=1M --numjobs=1 --iodepth=32 --runtime=180 --group_reporting --log_avg_msec=100 --time_based

14. Random Mixed
    sudo fio --name=throughtput-test --filename=/mnt/raid0/testfile --rw=randrw --rwmixread=80 --write_bw_log=bw --write_lat_log=lat --write_hist_log=hist --write_iops_log=iops --write_iolog=io --size=12000G --bs=1M --numjobs=1 --iodepth=32 --runtime=180 --group_reporting --log_avg_msec=100 --time_based

15. Sequential Mixes
    sudo fio --name=throughtput-test --filename=/mnt/raid0/testfile --rw=rw --rwmixread=80 --write_bw_log=bw --write_lat_log=lat --write_hist_log=hist --write_iops_log=iops --write_iolog=io --size=100G --bs=1M --numjobs=1 --iodepth=32 --runtime=180 --group_reporting --log_avg_msec=100 --time_based

awk '{print $1","$2}' bw_log.1.log > bw_log.1.csv
awk '{print $1","$2}' lat_log.1.log > lat_log.1.csv

16. Random Mixed
    sudo fio --name=throughtput-test --filename=/mnt/raid0/testfile --rw=randrw --rwmixread=80 --write_bw_log=bw --write_lat_log=lat --write_hist_log=hist --write_iops_log=iops --write_iolog=io --size=10G --bs=1M --numjobs=1 --iodepth=4 --runtime=180 --group_reporting --log_avg_msec=100 --time_based

17. Sequential Mixes
    sudo fio --name=throughtput-test --filename=/mnt/raid0/testfile --rw=rw --rwmixread=80 --write_bw_log=bw --write_lat_log=lat --write_hist_log=hist --write_iops_log=iops --write_iolog=io --size=10G --bs=1M --numjobs=1 --iodepth=4 --runtime=180 --group_reporting --log_avg_msec=100 --time_based

---

14. Sequential Read
    sudo fio --name=throughtput-test --filename=/mnt/raid0/testfile --rw=read --write_bw_log=bw --write_lat_log=lat --write_hist_log=hist --write_iops_log=iops --write_iolog=io --size=100G --bs=1M --numjobs=1 --iodepth=32 --runtime=180 --group_reporting --log_avg_msec=100 --time_based --
    sudo fio --name=throughput-test --filename=/mnt/raid0/testfile --rw=read --bs=1M --size=100G --numjobs=4 --iodepth=64 --direct=1 --runtime=180 --time_based --group_reporting --shared-log --log_avg_msec=100 --write_bw_log=bw --write_lat_log=lat --write_hist_log=hist --write_iops_log=iops --write_iolog=io
