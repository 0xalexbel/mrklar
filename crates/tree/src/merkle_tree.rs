use crate::error::MerkleTreeError;
use crate::pow2::two_pow_n;
use mrklar_common::merkle_proof::{MerkleProof, MerkleProofHash};
use serde::{Deserialize, Serialize};

const MAX_LEVEL_COUNT: u8 = 64;

#[derive(Debug, Serialize, Deserialize, Default)]
struct MerkleTreeLevel {
    level: u8,
    hashes: Vec<Vec<u8>>,
}

impl MerkleTreeLevel {
    pub fn new() -> Self {
        MerkleTreeLevel {
            level: 0,
            hashes: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() >= self.max_len()
    }

    pub fn max_len(&self) -> usize {
        MerkleTreeLevel::max_len_at_level(self.level)
    }

    pub fn max_len_at_level(level: u8) -> usize {
        two_pow_n(level) as usize
    }

    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    fn inc_level(&mut self) -> Result<u8, MerkleTreeError> {
        if self.level < MAX_LEVEL_COUNT {
            self.level += 1;
            Ok(self.level)
        } else {
            Err(MerkleTreeError::TooManyLevels)
        }
    }

    fn push_hash(&mut self, hash: Vec<u8>) -> Result<(), MerkleTreeError> {
        if self.len() >= self.max_len() {
            return Err(MerkleTreeError::LevelFull(self.level));
        }

        assert!(self.hashes.is_empty() || !self.hashes.last().unwrap().is_empty());
        self.hashes.push(hash);

        Ok(())
    }

    pub fn add_hash(&mut self, hash: Vec<u8>) -> Result<(), MerkleTreeError> {
        self.push_hash(hash)
    }

    pub fn sibling_index(&self, index: usize) -> usize {
        if index % 2 == 0 {
            index + 1
        } else {
            index - 1
        }
    }

    pub fn left_right_at(&self, index: usize) -> (usize, usize) {
        if index % 2 == 0 {
            (index, index + 1)
        } else {
            (index - 1, index)
        }
    }

    fn get_hash_at(&self, index: usize) -> Result<&Vec<u8>, MerkleTreeError> {
        if index >= self.len() {
            return Err(MerkleTreeError::NodeDoesNotExist(self.level, index));
        }

        let hash = &self.hashes[index];
        // should never happen
        assert!(!hash.is_empty());

        Ok(hash)
    }

    fn set_hash_at(&mut self, index: usize, hash: Vec<u8>) -> Result<(), MerkleTreeError> {
        if hash.is_empty() {
            return Err(MerkleTreeError::InvalidHash(self.level, index));
        }
        if index > self.len() {
            return Err(MerkleTreeError::NodeDoesNotExist(self.level, index));
        }
        if index == self.len() {
            self.push_hash(hash)
        } else {
            self.hashes[index] = hash;
            Ok(())
        }
    }

    pub fn try_parent_index(&self, index: usize) -> Result<usize, MerkleTreeError> {
        if self.level == 0 {
            return Err(MerkleTreeError::NodeDoesNotExist(self.level, index));
        }

        let parent_index = index / 2;
        if parent_index >= MerkleTreeLevel::max_len_at_level(self.level - 1) {
            return Err(MerkleTreeError::NodeDoesNotExist(
                self.level - 1,
                parent_index,
            ));
        }

        Ok(parent_index)
    }

    fn hash_left_right_at(&self, index: usize) -> Result<Vec<u8>, MerkleTreeError> {
        let (left, right) = self.left_right_at(index);
        assert!(left + 1 == right);

        if left >= self.len() {
            return Err(MerkleTreeError::NodeDoesNotExist(self.level, left));
        }

        let left_hash = self.get_hash_at(left)?;
        let right_hash = if right == self.len() {
            &MerkleProof::null_hash()
        } else {
            self.get_hash_at(right)?
        };

        Ok(MerkleProof::sha256_pair(left_hash, right_hash))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleTree {
    levels: Vec<MerkleTreeLevel>,
}

impl Default for MerkleTree {
    fn default() -> Self {
        MerkleTree {
            levels: vec![MerkleTreeLevel::new()],
        }
    }
}

impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree::default()
    }

    fn is_empty(&self) -> bool {
        self.level_count() == 1 && self.leaves().is_empty()
    }

    fn level_count(&self) -> u8 {
        assert!(!self.levels.is_empty());
        assert!(self.levels.len() < MAX_LEVEL_COUNT as usize);
        self.levels.len() as u8
    }

    fn level(&self, index: u8) -> &MerkleTreeLevel {
        assert!(!self.levels.is_empty());
        assert!(self.levels.len() < MAX_LEVEL_COUNT as usize);
        &self.levels[index as usize]
    }

    fn level_mut(&mut self, index: u8) -> &mut MerkleTreeLevel {
        assert!(!self.levels.is_empty());
        assert!(self.levels.len() < MAX_LEVEL_COUNT as usize);
        &mut self.levels[index as usize]
    }

    fn leaf_count(&self) -> usize {
        self.leaves().len()
    }

    fn leaves(&self) -> &MerkleTreeLevel {
        self.level(0)
    }

    fn leaves_mut(&mut self) -> &mut MerkleTreeLevel {
        self.level_mut(0)
    }

    fn root(&self) -> &MerkleTreeLevel {
        self.level(self.level_count() - 1)
    }

    /// Returns the merkle root 
    pub fn root_hash(&self) -> Result<&Vec<u8>, MerkleTreeError> {
        self.root().get_hash_at(0)
    }

    fn inc_leaves_level(&mut self) -> Result<(), MerkleTreeError> {
        for i in 0..self.levels.len() {
            let l = self.level_mut(i as u8);
            l.inc_level()?;
        }
        assert!(self.root().level == 1);

        self.levels.push(MerkleTreeLevel::new());

        assert!(self.root().level == 0);
        Ok(())
    }

    fn update_at(&mut self, index: usize) -> Result<(), MerkleTreeError> {
        let mut pos = index;

        for i in 0..(self.level_count() - 1) {
            let level = self.level(i);

            let hash = level.hash_left_right_at(pos)?;
            pos = level.try_parent_index(pos)?;

            let parent_level = self.level_mut(i + 1);
            parent_level.set_hash_at(pos, hash)?;
        }
        Ok(())
    }

    /// Add a new leaf to the merkle tree
    pub fn add_leaf(&mut self, hash: Vec<u8>) -> Result<usize, MerkleTreeError> {
        if self.leaves().is_full() || self.is_empty() {
            self.inc_leaves_level()?;
        }

        let leaves = self.leaves_mut();
        leaves.add_hash(hash)?;

        let new_leaf_index = leaves.len() - 1;

        self.update_at(new_leaf_index)?;

        Ok(new_leaf_index)
    }

    /// Compute the merkle proof of the leaf specified by `index`
    pub fn proof_at(&self, index: usize) -> Result<MerkleProof, MerkleTreeError> {
        if self.is_empty() {
            return Err(MerkleTreeError::TreeEmpty);
        }
        if index >= self.leaf_count() {
            return Err(MerkleTreeError::NodeDoesNotExist(
                self.leaves().level,
                index,
            ));
        }

        let mut proof: Vec<MerkleProofHash> = vec![];
        let mut pos = index;

        for i in 0..(self.level_count() - 1) {
            let level = self.level(i);

            let sibling_index = level.sibling_index(pos);
            if sibling_index > level.len() {
                return Err(MerkleTreeError::NodeDoesNotExist(i, sibling_index));
            }

            if sibling_index == level.len() {
                assert!(sibling_index == pos + 1);
                // sibling is a right node in the binary tree
                proof.push(MerkleProofHash::new_right(MerkleProof::null_hash()));
            } else if sibling_index == pos + 1 {
                // sibling is a right node in the binary tree
                proof.push(MerkleProofHash::new_right(
                    level.get_hash_at(sibling_index)?.clone(),
                ));
            } else {
                // sibling is a left node in the binary tree
                proof.push(MerkleProofHash::new_left(
                    level.get_hash_at(sibling_index)?.clone(),
                ));
            }

            pos = level.try_parent_index(pos)?;
            assert!(level.try_parent_index(sibling_index)? == pos);
        }

        Ok(MerkleProof::with_hashes(proof))
    }
}

#[cfg(test)]
mod test {
    use super::MerkleTree;

    #[test]
    fn test_empty() {
        let t = MerkleTree::new();
        assert!(t.is_empty());

        // only 1 level
        assert_eq!(t.level_count(), 1);

        // no leaf
        assert_eq!(t.leaf_count(), 0);

        // root hash must be null hash
        let root_hash = t.root_hash();
        assert!(root_hash.is_err());

        // proof does not exist
        let proof = t.proof_at(0);
        assert!(proof.is_err());

        // proof does not exist
        let proof = t.proof_at(1);
        assert!(proof.is_err());
    }

    #[test]
    fn test_1() {
        let mut t = MerkleTree::new();

        let left_hex = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        //let right_hex = "0000000000000000000000000000000000000000000000000000000000000000";
        let root_hex = "ce4c6ed23866d28bd42cf36eaf84076e91501bcbee5b6cff3ecbf00070383d6d";

        let left = hex::decode(left_hex).unwrap();
        let root = hex::decode(root_hex).unwrap();

        t.add_leaf(left.clone()).unwrap();

        let root_hash = t.root_hash().unwrap();
        assert_eq!(root, *root_hash);

        // proof at 0 should be ok
        let proof = t.proof_at(0).unwrap();
        let verified = proof.verify(&left, root_hash);
        assert!(verified);

        // proof at 1 does not exist
        let proof = t.proof_at(1);
        assert!(proof.is_err());
    }

    #[test]
    fn test_2() {
        let mut t = MerkleTree::new();

        let left_hex = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        let right_hex = "1c27ae443e93ef623d8670b611ae1d7f7d71c7f103258ff8ce0c90fab557dfd8";
        let root_hex = "5485e2e93b173cbe9abfce3d738ff80d444daa9b1e1717551bbd599bb2d4a78c";

        let left = hex::decode(left_hex).unwrap();
        let right = hex::decode(right_hex).unwrap();
        let root = hex::decode(root_hex).unwrap();

        t.add_leaf(left.clone()).unwrap();
        t.add_leaf(right.clone()).unwrap();

        let root_hash = t.root_hash().unwrap();
        assert_eq!(root, *root_hash);

        // proof at 0 should be ok
        let proof = t.proof_at(0).unwrap();
        let verified = proof.verify(&left, root_hash);
        assert!(verified);

        // proof at 1 should be ok
        let proof = t.proof_at(1).unwrap();
        let verified = proof.verify(&right, root_hash);
        assert!(verified);
    }

    #[test]
    fn test_3() {
        let mut t = MerkleTree::new();

        let dead_hex = "deadbeeff1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        let a_hex = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        let b_hex = "1c27ae443e93ef623d8670b611ae1d7f7d71c7f103258ff8ce0c90fab557dfd8";
        let c_hex = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        //let d_hex = "0000000000000000000000000000000000000000000000000000000000000000";

        //let ab_hex = "5485e2e93b173cbe9abfce3d738ff80d444daa9b1e1717551bbd599bb2d4a78c";
        //let cd_hex = "ce4c6ed23866d28bd42cf36eaf84076e91501bcbee5b6cff3ecbf00070383d6d";

        let root_hex = "0c56afbc57fe3c70f0aa21050111c5adb6a65bd51edef7cf5411e28a0076f6da";

        let a = hex::decode(a_hex).unwrap();
        let b = hex::decode(b_hex).unwrap();
        let c = hex::decode(c_hex).unwrap();
        let root = hex::decode(root_hex).unwrap();

        t.add_leaf(a.clone()).unwrap();
        t.add_leaf(b.clone()).unwrap();
        t.add_leaf(c.clone()).unwrap();

        let root_hash = t.root_hash().unwrap();
        assert_eq!(root, *root_hash);

        // proof at 0 should be ok
        let proof = t.proof_at(0).unwrap();
        let verified = proof.verify(&a, root_hash);
        assert!(verified);

        // proof at 1 should be ok
        let proof = t.proof_at(1).unwrap();
        let verified = proof.verify(&b, root_hash);
        assert!(verified);

        // proof at 2 should be ok
        let proof = t.proof_at(2).unwrap();
        let verified = proof.verify(&c, root_hash);
        assert!(verified);

        // should fail with garbage
        let dead = hex::decode(dead_hex).unwrap();
        let verified = proof.verify(&dead, root_hash);
        assert!(!verified);
    }

    #[test]
    fn test_4() {
        let mut t = MerkleTree::new();

        let a_hex = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        let b_hex = "1c27ae443e93ef623d8670b611ae1d7f7d71c7f103258ff8ce0c90fab557dfd8";
        let c_hex = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        let d_hex = "1c27ae443e93ef623d8670b611ae1d7f7d71c7f103258ff8ce0c90fab557dfd8";

        // let ab_hex = "5485e2e93b173cbe9abfce3d738ff80d444daa9b1e1717551bbd599bb2d4a78c";
        // let cd_hex = "5485e2e93b173cbe9abfce3d738ff80d444daa9b1e1717551bbd599bb2d4a78c";

        let root_hex = "339fe1a625ad60d5680bd37627c53414ea118b67dde0b8eabbb585547a024342";

        let a = hex::decode(a_hex).unwrap();
        let b = hex::decode(b_hex).unwrap();
        let c = hex::decode(c_hex).unwrap();
        let d = hex::decode(d_hex).unwrap();

        let root = hex::decode(root_hex).unwrap();

        t.add_leaf(a).unwrap();
        t.add_leaf(b).unwrap();
        t.add_leaf(c).unwrap();
        t.add_leaf(d).unwrap();

        let root_hash = t.root_hash().unwrap();
        assert_eq!(root, *root_hash);
    }

    #[test]
    fn test_5() {
        let mut t = MerkleTree::new();

        let a_hex = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        let b_hex = "1c27ae443e93ef623d8670b611ae1d7f7d71c7f103258ff8ce0c90fab557dfd8";
        let c_hex = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        let d_hex = "1c27ae443e93ef623d8670b611ae1d7f7d71c7f103258ff8ce0c90fab557dfd8";
        let e_hex = "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb";
        //let f_hex = "0000000000000000000000000000000000000000000000000000000000000000";

        // let ab_hex = "5485e2e93b173cbe9abfce3d738ff80d444daa9b1e1717551bbd599bb2d4a78c";
        // let cd_hex = "5485e2e93b173cbe9abfce3d738ff80d444daa9b1e1717551bbd599bb2d4a78c";
        // let ef_hex = "ce4c6ed23866d28bd42cf36eaf84076e91501bcbee5b6cff3ecbf00070383d6d";
        // let gh_hex = "0000000000000000000000000000000000000000000000000000000000000000";

        // let abcd_hex = "339fe1a625ad60d5680bd37627c53414ea118b67dde0b8eabbb585547a024342";
        // let efgh_hex = "9f92c847b67c5f4619058a59ef25e9ef9339b6776f2db12a08bba729f43727ac";

        let root_hex = "cda278afb1adc0fbf06c52bbbf9e535f1d95c072a52c6e170cdfa3c63d55d378";

        let a = hex::decode(a_hex).unwrap();
        let b = hex::decode(b_hex).unwrap();
        let c = hex::decode(c_hex).unwrap();
        let d = hex::decode(d_hex).unwrap();
        let e = hex::decode(e_hex).unwrap();
        let root = hex::decode(root_hex).unwrap();

        t.add_leaf(a.clone()).unwrap();
        t.add_leaf(b.clone()).unwrap();
        t.add_leaf(c.clone()).unwrap();
        t.add_leaf(d.clone()).unwrap();
        t.add_leaf(e.clone()).unwrap();

        let root_hash = t.root_hash().unwrap();
        assert_eq!(root, *root_hash);

        // proof at 0 should be ok
        let proof = t.proof_at(0).unwrap();
        let verified = proof.verify(&a, root_hash);
        assert!(verified);

        // proof at 1 should be ok
        let proof = t.proof_at(1).unwrap();
        let verified = proof.verify(&b, root_hash);
        assert!(verified);

        // proof at 2 should be ok
        let proof = t.proof_at(2).unwrap();
        let verified = proof.verify(&c, root_hash);
        assert!(verified);

        // proof at 3 should be ok
        let proof = t.proof_at(3).unwrap();
        let verified = proof.verify(&d, root_hash);
        assert!(verified);

        // proof at 4 should be ok
        let proof = t.proof_at(4).unwrap();
        let verified = proof.verify(&e, root_hash);
        assert!(verified);
    }

    fn rand_hash() -> Vec<u8> {
        let mut v = vec![];
        for _ in 0..32 {
            v.push(rand::random::<u8>())
        }
        v
    }

    #[test]
    fn test_1000() {
        let mut t = MerkleTree::new();

        let n = 1000;

        let mut rand_hashes: Vec<Vec<u8>> = vec![];
        for _ in 0..n {
            rand_hashes.push(rand_hash());
        }
        assert_eq!(rand_hashes.len(), n);

        rand_hashes.iter().for_each(|h| {
            t.add_leaf(h.clone()).unwrap();
        });
        assert_eq!(t.leaf_count(), n);

        let root_hash = t.root_hash().unwrap();

        // verify valid hashes
        rand_hashes.iter().enumerate().for_each(|(i, h)| {
            let proof = t.proof_at(i).unwrap();
            let verified = proof.verify(h, root_hash);
            assert!(verified);
        });

        // verify invalid hashes
        rand_hashes.iter().enumerate().for_each(|(i, h)| {
            let mut garbage_h = h.clone();
            if garbage_h[0] == 0 {
                garbage_h[0] = 1;
            } else {
                garbage_h[0] = 0;
            }
            let proof = t.proof_at(i).unwrap();
            let verified = proof.verify(&garbage_h, root_hash);
            assert!(!verified);
        });
        println!("levels={}", t.level_count());
        println!("leaves={}", t.leaf_count());
    }
}
