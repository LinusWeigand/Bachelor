sudo yum install -y mdadm xfsprogs fio
sudo mdadm --create /dev/md0 --level=5 --raid-devices=2 /dev/nvme1n1 /dev/nvme2n1
sudo mkfs.xfs /dev/md0
sudo mkdir /mnt/raid0
sudo mount /dev/md0 /mnt/raid0
echo '/dev/md0 /mnt/raid0 xfs defaults,noatime 0 0' | sudo tee -a /etc/fstab

