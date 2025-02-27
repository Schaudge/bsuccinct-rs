use csf::coding::{BuildMinimumRedundancy, minimum_redundancy};
use csf::fp::{OptimalLevelSize, ProportionalLevelSize, ResizedLevel};
use ph::BuildSeededHasher;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use csf::coding::minimum_redundancy::BitsPerFragment;
use csf::{fp, ls, GetSize};

pub trait CSFBuilder {
    const CAN_DETECT_ABSENCE: bool = true;
    type CSF: GetSize;
    fn new(self, keys: &[u32], values: &[u32], frequencies: HashMap::<u32, u32>) -> Self::CSF;
    fn value(f: &Self::CSF, k: u32, levels: &mut u64) -> Option<u32>;
}

pub trait PrintParams {
    fn print_params(&self, file: &mut Option<File>);
}

impl PrintParams for OptimalLevelSize {
    fn print_params(&self, file: &mut Option<File>) {
        print!(" optim");
        if let Some(ref mut f) = file {
            write!(f, " true 100").unwrap();
        }
    }
}

impl PrintParams for ResizedLevel<OptimalLevelSize> {
    fn print_params(&self, file: &mut Option<File>) {
        print!(" optim*{}%", self.percent);
        if let Some(ref mut f) = file {
            write!(f, " true {}", self.percent).unwrap();
        }
    }
}

impl PrintParams for ProportionalLevelSize {
    fn print_params(&self, file: &mut Option<File>) {
        print!(" levels {}%", self.percent);
        if let Some(ref mut f) = file {
            write!(f, " false {}", self.percent).unwrap();
        }
    }
}

/*impl<LSC, CSB, S> CSFBuilder for fp::MapConf<LSC, CSB, S>
where CSB: fp::CollisionSolverBuilder, S: BuildSeededHasher
 {
    type CSF = fp::Map<S>;

    fn new(&self, keys: &[u32], values: &[u32], coding: Coding<u32>) -> Self::CSF {
        Self::CSF::from_slices_with_conf(
            keys.to_owned().as_mut(), values,
            self.clone(),
            &mut ())
    }

    #[inline(always)] fn value(f: &Self::CSF, k: u32, levels: &mut u64) -> Option<u32> {
        f.get_stats(&k, levels).unwrap()
    }
}*/

impl<LSC, CSB, S> CSFBuilder for fp::CMapConf<BuildMinimumRedundancy, LSC, CSB, S>
where LSC: fp::LevelSizeChooser, CSB: fp::CollisionSolverBuilder+fp::IsLossless, S: BuildSeededHasher
 {
    type CSF = fp::CMap<minimum_redundancy::Coding<u32>, S>;

    fn new(self, keys: &[u32], values: &[u32], frequencies: HashMap::<u32, u32>) -> Self::CSF {
        Self::CSF::from_slices_with_coding_conf(
            keys.to_owned().as_mut(), values,
            minimum_redundancy::Coding::<u32, _>::from_frequencies(BitsPerFragment(self.coding.bits_per_fragment), frequencies),
            self,
            &mut ())
    }

    #[inline(always)] fn value(f: &Self::CSF, k: u32, levels: &mut u64) -> Option<u32> {
        f.get_stats(&k, levels).copied()
    }
}

pub const FP_HEADER: &'static str = "bits/fragment level_size_optimal level_size_percent";

impl<LSC, CSB, S> PrintParams for fp::CMapConf<BuildMinimumRedundancy, LSC, CSB, S>
where LSC: PrintParams, CSB: fp::CollisionSolverBuilder, S: BuildSeededHasher {
    fn print_params(&self, file: &mut Option<File>) {
        if let Some(ref mut f) = file {
            write!(f, " {}", self.coding.bits_per_fragment).unwrap();
        }
        print!("fp");
        self.level_size_chooser.print_params(file);
        print!(" {} b/frag: ", self.coding.bits_per_fragment);
    }
}

impl<LSC, GS, SS, S> CSFBuilder for fp::GOCMapConf<BuildMinimumRedundancy, LSC, GS, SS, S>
where LSC: fp::LevelSizeChooser, GS: fp::GroupSize, SS: fp::SeedSize, S: BuildSeededHasher
{
    type CSF = fp::GOCMap<minimum_redundancy::Coding<u32>, GS, SS, S>;

    fn new(self, keys: &[u32], values: &[u32], frequencies: HashMap::<u32, u32>) -> Self::CSF {
        Self::CSF::from_slices_with_coding_conf(
            keys.to_owned().as_mut(), values,
            minimum_redundancy::Coding::<u32, _>::from_frequencies(BitsPerFragment(self.coding.bits_per_fragment), frequencies),
            self,
            &mut ())
    }

    #[inline(always)] fn value(f: &Self::CSF, k: u32, levels: &mut u64) -> Option<u32> {
        f.get_stats(&k, levels).copied()
    }
}

pub const FPGO_HEADER: &'static str = "bits/fragment bits/seed bits/group level_size_optimal level_size_percent";

impl<LSC, GS, SS, S> PrintParams for fp::GOCMapConf<BuildMinimumRedundancy, LSC, GS, SS, S>
where LSC: PrintParams, GS: fp::GroupSize, SS: fp::SeedSize, S: BuildSeededHasher {
    fn print_params(&self, file: &mut Option<File>) {
        let (bits_per_seed, bits_per_group): (u8, u8) = (self.goconf.bits_per_seed.into(), self.goconf.bits_per_group.into());
        if let Some(ref mut f) = file {
            write!(f, " {} {} {}", self.coding.bits_per_fragment, bits_per_seed, bits_per_group).unwrap();
        }
        print!("fpgo");
        self.level_size_chooser.print_params(file);
        print!(" {} b/seed {} b/group {} b/frag: ", bits_per_seed, bits_per_group, self.coding.bits_per_fragment);
    }
}

/// Build `ls::CMap` with given number of bits per code fragment.
pub struct BuildLSCMap(pub u8);

impl CSFBuilder for BuildLSCMap
{
    type CSF = ls::CMap<minimum_redundancy::Coding<u32>>;

    fn new(self, keys: &[u32], values: &[u32], frequencies: HashMap::<u32, u32>) -> Self::CSF {
        Self::CSF::try_from_kv_with_coding_conf(keys, values,
             minimum_redundancy::Coding::<u32, _>::from_frequencies(BitsPerFragment(self.0), frequencies),
             ls::MapConf::new(),
             0).unwrap()
    }

    #[inline(always)] fn value(f: &Self::CSF, k: u32, levels: &mut u64) -> Option<u32> {
        f.get_stats(&k, levels).copied()
    }
}

pub const LS_HEADER: &'static str = "bits/fragment";

impl PrintParams for BuildLSCMap {
    fn print_params(&self, file: &mut Option<File>) {
        print!("ls {} b/frag: ", self.0);
        if let Some(ref mut f) = file {
            write!(f, " {}", self.0).unwrap();
        }
    }
}
