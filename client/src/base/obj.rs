use crate::*;
use bytemuck::{cast_slice, NoUninit};
use glow::{
    Context, HasContext, NativeBuffer, NativeVertexArray, ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER,
    FLOAT, STATIC_DRAW, TRIANGLES, TRIANGLE_STRIP, UNSIGNED_BYTE,
};
use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Copy, Debug)]
pub struct Buffers {
    vao: NativeVertexArray,
    vbo: NativeBuffer,
    ebo: NativeBuffer,
}

impl Buffers {
    pub const fn vao(&self) -> NativeVertexArray {
        self.vao
    }

    pub const fn vbo(&self) -> NativeBuffer {
        self.vbo
    }

    pub const fn ebo(&self) -> NativeBuffer {
        self.ebo
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Object {
    program: Program,
    buffers: Buffers,
    data: ObjectData,
    mode: u32,
    element_type: u32,
    len: i32,
}

impl Object {
    const fn new(
        program: Program,
        buffers: Buffers,
        mode: u32,
        element_type: u32,
        len: i32,
        data: ObjectData,
    ) -> Self {
        Self {
            program,
            buffers,
            data,
            mode,
            element_type,
            len,
        }
    }

    /// Construct a simple cube (8 vertices; 14 indices).
    ///
    /// Explanation: https://stackoverflow.com/a/79336923/13449866
    pub fn create_flat_cube(
        gl: &Context,
        program: Program,
        pos: Vector,
        dim: Vector,
        color: Color,
        id: Id,
        kind: RawObjectDataUnit,
    ) -> Result<Self> {
        let raw_data = match kind {
            RawObjectDataUnit::Player => RawObjectData::Player(PlayerData::new(pos)),
            RawObjectDataUnit::Basic => RawObjectData::Basic(BasicData::new(pos, dim)),
        };
        let data = ObjectData::new(id, color, raw_data);

        Self::create_flat_cube_with(gl, program, data)
    }

    /// Construct a simple cube (8 vertices; 14 indices) with specified [`ObjectData`].
    ///
    /// Explanation: https://stackoverflow.com/a/79336923/13449866
    pub fn create_flat_cube_with(gl: &Context, program: Program, data: ObjectData) -> Result<Self> {
        let x = -1.0;
        let y = -1.0;
        let z = -1.0;

        let xw = 1.0;
        let yh = 1.0;
        let zd = 1.0;

        #[rustfmt::skip]
        let vertices = [
           xw,  yh,   z,  // [1, 1, 0] [00]
            x,  yh,   z,  // [0, 1, 0] [01]
           xw,  yh,  zd,  // [1, 1, 1] [02]
            x,  yh,  zd,  // [0, 1, 1] [03]
           xw,   y,   z,  // [1, 0, 0] [04]
            x,   y,   z,  // [0, 0, 0] [05]
            x,   y,  zd,  // [0, 0, 1] [06]
           xw,   y,  zd,  // [1, 0, 1] [07]
        ];

        #[rustfmt::skip]
        let indices = [
            0, 1, 4, 5, 6, 1, 3, 0, 2, 4, 7, 6, 2, 3
        ];

        Self::from_raw::<f32, u8>(
            gl,
            program,
            vertices.as_slice(),
            indices.as_slice(),
            TRIANGLE_STRIP,
            UNSIGNED_BYTE,
            data,
            false,
        )
    }

    /// Construct a normal cube (24 vertices; 36 indices).
    ///
    /// Explanation: https://stackoverflow.com/a/79337030/13449866
    pub fn create_cube(
        gl: &Context,
        program: Program,
        pos: Vector,
        dim: Vector,
        color: Color,
        id: Id,
        kind: RawObjectDataUnit,
    ) -> Result<Self> {
        let raw_data = match kind {
            RawObjectDataUnit::Player => RawObjectData::Player(PlayerData::new(pos)),
            RawObjectDataUnit::Basic => RawObjectData::Basic(BasicData::new(pos, dim)),
        };
        let data = ObjectData::new(id, color, raw_data);

        match program.kind() {
            ProgramUnit::Simple => Self::create_flat_cube_with(gl, program, data),
            ProgramUnit::Normal => Self::create_cube_with(gl, program, data),
        }
    }

    /// Construct a normal cube (24 vertices; 36 indices) with specified [`ObjectData`].
    ///
    /// Explanation: https://stackoverflow.com/a/79337030/13449866
    pub fn create_cube_with(gl: &Context, program: Program, data: ObjectData) -> Result<Self> {
        let x = -1.0;
        let y = -1.0;
        let z = -1.0;

        let xw = 1.0;
        let yh = 1.0;
        let zd = 1.0;

        #[rustfmt::skip]
        let vertices = [
             // BACK
             x,   y,   z,  /* [0, 0, 0] */   0.0,  0.0, -1.0,  //  [00]
             x,  yh,   z,  /* [0, 1, 0] */   0.0,  0.0, -1.0,  //  [01]
            xw,   y,   z,  /* [1, 0, 0] */   0.0,  0.0, -1.0,  //  [02]
            xw,  yh,   z,  /* [1, 1, 0] */   0.0,  0.0, -1.0,  //  [03]

             // FRONT
             x,   y,  zd,  /* [0, 0, 1] */   0.0,  0.0,  1.0,  //  [04]
             x,  yh,  zd,  /* [0, 1, 1] */   0.0,  0.0,  1.0,  //  [05]
            xw,   y,  zd,  /* [1, 0, 1] */   0.0,  0.0,  1.0,  //  [06]
            xw,  yh,  zd,  /* [1, 1, 1] */   0.0,  0.0,  1.0,  //  [07]

             // LEFT
             x,   y,  zd,  /* [0, 0, 1] */  -1.0,  0.0,  0.0,  //  [08]
             x,  yh,  zd,  /* [0, 1, 1] */  -1.0,  0.0,  0.0,  //  [09]
             x,   y,   z,  /* [0, 0, 0] */  -1.0,  0.0,  0.0,  //  [10]
             x,  yh,   z,  /* [0, 1, 0] */  -1.0,  0.0,  0.0,  //  [11]

             // RIGHT
             xw,   y,  zd,  /* [1, 0, 1] */  1.0,  0.0,  0.0,  //  [12]
             xw,  yh,  zd,  /* [1, 1, 1] */  1.0,  0.0,  0.0,  //  [13]
             xw,   y,   z,  /* [1, 0, 0] */  1.0,  0.0,  0.0,  //  [14]
             xw,  yh,   z,  /* [1, 1, 0] */  1.0,  0.0,  0.0,  //  [15]

             // TOP
              x,  yh,   z,  /* [0, 1, 0] */  0.0,  1.0,  0.0,  //  [16]
              x,  yh,  zd,  /* [0, 1, 1] */  0.0,  1.0,  0.0,  //  [17]
             xw,  yh,   z,  /* [1, 1, 0] */  0.0,  1.0,  0.0,  //  [18]
             xw,  yh,  zd,  /* [1, 1, 1] */  0.0,  1.0,  0.0,  //  [19]

             // BOTTOM
              x,   y,   z,  /* [0, 0, 0] */  0.0, -1.0,  0.0,  //  [20]
              x,   y,  zd,  /* [0, 0, 1] */  0.0, -1.0,  0.0,  //  [21]
             xw,   y,   z,  /* [1, 0, 0] */  0.0, -1.0,  0.0,  //  [22]
             xw,   y,  zd,  /* [1, 0, 1] */  0.0, -1.0,  0.0,  //  [23]
        ];

        #[rustfmt::skip]
        let indices = [
            // FRONT
             0,  3,  2,    1,  3,  0,

            // BACK
             6,  7,  4,    4,  7,  5,

            // LEFT
             8, 11, 10,    9, 11,  8,

            // RIGHT
            14, 15, 12,   12, 15, 13,

            // TOP
            16, 19, 18,   17, 19, 16,

            // BOTTOM
            22, 23, 20,   20, 23, 21,
        ];

        Self::from_raw::<f32, u8>(
            gl,
            program,
            vertices.as_slice(),
            indices.as_slice(),
            TRIANGLES,
            UNSIGNED_BYTE,
            data,
            true,
        )
    }

    pub fn from_raw<V: NoUninit, I: NoUninit>(
        gl: &Context,
        program: Program,
        vertices: &[V],
        indices: &[I],
        mode: u32,
        element_type: u32,
        mut data: ObjectData,
        has_norms: bool,
    ) -> Result<Self> {
        unsafe {
            // creates and bind Vertex Array Object (VAO)
            let vao = gl.create_vertex_array()?;
            let vbo = gl.create_buffer()?;
            let ebo = gl.create_buffer()?;

            let mut stride = 3;

            if has_norms {
                stride += 3
            }

            gl.bind_vertex_array(Some(vao));

            // create and bind Vertex Buffer Object (VBO)
            gl.bind_buffer(ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, cast_slice(vertices), STATIC_DRAW);

            // create and bind Elements Buffer Object (EBO)
            gl.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(ELEMENT_ARRAY_BUFFER, cast_slice(indices), STATIC_DRAW);

            // enable `pos` attribute
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, FLOAT, false, stride * size_of::<f32>() as i32, 0);

            if has_norms {
                // enable `norm` attribute
                gl.enable_vertex_attrib_array(1);
                gl.vertex_attrib_pointer_f32(
                    1,
                    3,
                    FLOAT,
                    false,
                    stride * size_of::<f32>() as i32,
                    3 * size_of::<f32>() as i32,
                );
            }

            // unbind buffers
            gl.bind_vertex_array(None);
            gl.bind_buffer(ARRAY_BUFFER, None);
            gl.bind_buffer(ELEMENT_ARRAY_BUFFER, None);

            let buf = Buffers { vao, vbo, ebo };

            // initial transformation update
            data.model_upt();

            Ok(Self::new(
                program,
                buf,
                mode,
                element_type,
                indices.len() as i32,
                data,
            ))
        }
    }

    pub const fn program(&self) -> Program {
        self.program
    }

    pub const fn buffers(&self) -> Buffers {
        self.buffers
    }

    pub const fn data(&self) -> &ObjectData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut ObjectData {
        &mut self.data
    }

    pub const fn vao(&self) -> NativeVertexArray {
        self.buffers.vao()
    }

    pub const fn vbo(&self) -> NativeBuffer {
        self.buffers.vbo()
    }

    pub const fn ebo(&self) -> NativeBuffer {
        self.buffers.ebo()
    }

    pub const fn id(&self) -> Id {
        self.data.id()
    }

    pub const fn color(&self) -> &[f32] {
        self.data.color()
    }

    pub const fn mode(&self) -> u32 {
        self.mode
    }

    pub const fn element_type(&self) -> u32 {
        self.element_type
    }

    pub const fn len(&self) -> i32 {
        self.len
    }
}

impl Deref for Object {
    type Target = ObjectData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Clone, Debug, Default)]
pub struct RawObjects {
    opaque: HashMap<Id, Object>,
}

impl RawObjects {
    /// create and add a new cube with specified attributes.
    pub fn new_cube(
        &mut self,
        gl: &Context,
        id: Id,
        program: Program,
        pos: Vector,
        dim: Vector,
        color: Color,
        kind: RawObjectDataUnit,
    ) -> Result {
        let raw_data = match kind {
            RawObjectDataUnit::Player => RawObjectData::Player(PlayerData::new(pos)),
            RawObjectDataUnit::Basic => RawObjectData::Basic(BasicData::new(pos, dim)),
        };
        let data = ObjectData::new(id, color, raw_data);

        self.new_cube_with(gl, program, data)
    }

    /// create and add a new cube with specified [`ObjectData`].
    pub fn new_cube_with(&mut self, gl: &Context, program: Program, data: ObjectData) -> Result {
        let obj = Object::create_cube_with(gl, program, data)?;
        self.opaque.insert(data.id(), obj);
        Ok(())
    }

    /// return a mutable reference of the specified object.
    pub fn get_mut(&mut self, id: Id) -> Option<&mut ObjectData> {
        self.opaque.get_mut(&id).map(Object::data_mut)
    }

    /// insert a new object.
    pub fn insert(&mut self, obj: Object) {
        self.opaque.insert(obj.id(), obj);
    }

    /// create and add a new light (simple shading with color as light color) object.
    pub fn new_light(
        &mut self,
        gl: &Context,
        id: Id,
        program: Program,
        pos: Vector,
        dim: Vector,
        color: Color,
    ) -> Result {
        let obj =
            Object::create_flat_cube(gl, program, pos, dim, color, id, RawObjectDataUnit::Basic)?;
        self.opaque.insert(id, obj);
        Ok(())
    }

    /// remove the object specified object.
    pub fn remove(&mut self, id: Id) -> Option<Object> {
        self.opaque.remove(&id)
    }

    /// retain only the objects specified by object type.
    pub fn retain(&mut self, gl: &Context, kind: RawObjectDataUnit) {
        self.opaque.retain(|_, obj| {
            if kind == obj.kind() {
                free_buffers(gl, obj.buffers());
                false
            } else {
                true
            }
        });
    }

    /// return an iterator of every light object
    pub fn lights(&self) -> impl Iterator<Item = &Object> {
        self.opaque.values().filter(|o| o.is_light())
    }

    /// return an iterator of every object in descending order,
    /// based on the alpha value color of each object.
    pub fn iter(&self) -> impl Iterator<Item = &Object> {
        self.opaque.values()
    }
}
