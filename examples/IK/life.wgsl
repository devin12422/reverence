
struct LifeParams {
    width : u32,
    height : u32,
    threshold : f32,
};

struct Cells {
    cells : array<u32>,
};

@group(0) @binding(0) var<uniform> params : LifeParams;
@group(0) @binding(1) var<storage,read> cellSrc :  Cells;
@group(0) @binding(2) var<storage,read_write> cellDst :  Cells;

@compute @workgroup_size(8, 8)
fn life(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let X : u32 = global_id.x;
    let Y : u32 = global_id.y;
    let W : u32 = params.width;
    let H : u32 = params.height;

    if (X > W || Y > H) {
        return;
    }

    var count : i32 = 0;

    let pix : u32 = Y * W + X;
    for (var y  = Y - 1u; y <= Y + 1u; y = y + 1u) {
        for (var x = X - 1u; x <= X + 1u; x = x + 1u) {
            if(y != Y && x != X){
                count = count + i32(cellSrc.cells[y * W + x]);
            }
        }
    }

    let ov : u32 = cellSrc.cells[pix];
    let was_alive : bool = ov  == u32(1);
    var nv : u32;

    // in the first clause, "3 or 4" includes the center cell
    if (was_alive && (count == 2 || count == 3)) {
        nv = u32(1);
    } else {
        if (!was_alive && count == 3) {
            nv = u32(1);
        } else {
            nv = u32(0);
        }
    }

    cellDst.cells[pix] = nv;

   }
