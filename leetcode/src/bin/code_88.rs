pub fn merge(nums1: &mut Vec<i32>, m: i32, nums2: &mut Vec<i32>, n: i32) {
    let mut len = m + n;
    let mut x = m;
    let mut y = n;
    while x > 0 && y > 0 {
        if  nums2[y as usize -1] > nums1[x as usize-1] {
            nums1[len as usize-1] = nums2[y as usize-1];
            y -= 1;
        } else {
            nums1[len as usize-1] = nums1[x as usize-1];
            x -= 1;
        }
        len -= 1;
    }
    while y > 0 {
        nums1[len as usize-1] = nums2[y as usize-1];
        y -= 1;
        len -= 1;
    }
}
fn main() {
    let mut nums1 = vec![4,5,6,0,0,0];
    let mut nums2 = vec![1,2,3];
    merge(&mut nums1, 3, &mut nums2, 3);
    assert_eq!(nums1, vec![1,2,3,4,5,6]);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge() {
        let mut nums1 = vec![1,2,3,0,0,0];
        let mut nums2 = vec![2,5,6];
        merge(&mut nums1, 3, &mut nums2, 3);
        assert_eq!(nums1, vec![1,2,2,3,5,6]);
    }
}