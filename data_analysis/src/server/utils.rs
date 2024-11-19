use std::error::Error;

pub trait FromParts: Sized {
    fn from_parts(parts: &[&str]) -> Result<Self, Box<dyn Error>>;
}

#[derive(Clone, Debug)]
pub struct CPU {
    pub user: f64,
    pub nice: f64,
    pub system: f64,
    pub idle: f64,
    pub iowait: f64,
    pub irq: f64,
    pub softirq: f64,
}

impl FromParts for CPU {
    fn from_parts(parts: &[&str]) -> Result<Self, Box<dyn Error>> {
        if parts.len() < 7 {
            return Err("Insufficient CPU parts".into());
        }

        Ok(CPU {
            user: parts[0].trim().parse::<f64>()?,
            nice: parts[1].trim().parse::<f64>()?,
            system: parts[2].trim().parse::<f64>()?,
            idle: parts[3].trim().parse::<f64>()?,
            iowait: parts[4].trim().parse::<f64>()?,
            irq: parts[5].trim().parse::<f64>()?,
            softirq: parts[6].trim().parse::<f64>()?,
        })
    }
}

#[derive(Clone)]
pub struct RAM {
    pub total: f64,
    pub used: f64,
    pub free: f64,
    pub available: f64,
}

impl FromParts for RAM {
    fn from_parts(parts: &[&str]) -> Result<Self, Box<dyn Error>> {
        if parts.len() < 4 {
            return Err("Insuffiecient RAM parts".into());
        }

        Ok(RAM {
            total: parts[0].trim().parse::<f64>()?,
            used: parts[1].trim().parse::<f64>()?,
            free: parts[2].trim().parse::<f64>()?,
            available: parts[3].trim().parse::<f64>()?,
        })
    }
}

#[derive(Clone)]
pub struct SOFTIRQ {
    pub total: f64,
    pub hi: f64,
    pub timer: f64,
    pub net_tx: f64,
    pub net_tr: f64,
    pub block: f64,
    pub irq_poll: f64,
    pub tasklet: f64,
    pub sched: f64,
    pub hrtimer: f64,
    pub rcu: f64,
}

impl FromParts for SOFTIRQ {
    fn from_parts(parts: &[&str]) -> Result<Self, Box<dyn Error>> {
        if parts.len() < 11 {
            return Err("Insuffiecient SOFIRQ parts".into());
        }

        Ok(SOFTIRQ {
            total: parts[0].trim().parse::<f64>()?,
            hi: parts[1].trim().parse::<f64>()?,
            timer: parts[2].trim().parse::<f64>()?,
            net_tx: parts[3].trim().parse::<f64>()?,
            net_tr: parts[4].trim().parse::<f64>()?,
            block: parts[5].trim().parse::<f64>()?,
            irq_poll: parts[6].trim().parse::<f64>()?,
            tasklet: parts[7].trim().parse::<f64>()?,
            sched: parts[8].trim().parse::<f64>()?,
            hrtimer: parts[9].trim().parse::<f64>()?,
            rcu: parts[10].trim().parse::<f64>()?,
        })
    }
}
