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

    static is(arr: string[], cmd: string, f: {(rest: string[]): void}) {
        let parts = cmd.split(":");
        for (let i = 0; i < parts.length; i++) {
            if (parts[i] !== arr[i]) {
                return;
            }
        }
        f(arr.slice(arr.length));
    }
}