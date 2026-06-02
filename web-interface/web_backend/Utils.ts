export class Utils {
    static contains(arr: number[][], val: number[]) {
        return arr.some(item => {
            for (let i = 0; i < item.length; i++) {
                if (item[i] !== val[i]) {
                    return false;
                }
            }
            return true;
        });
    }
}