// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file contains an abstraction of concrete *access paths*, which are canonical names for a particular cell in
//! memory. Some examples of concrete paths are:
//! * `0x7/M::T/f` (i.e., the field `f` of the `M::T` resource stored at address `0x7`
//! * `Formal(0)/[2]` (i.e., the value stored at index 2 of the array bound the 0th formal of the current procedure)
//! An abstract path is similar; it consists of the following components:
//! * A *root*, which is either an abstract address or a local
//! * Zero or more *offsets*, where an offset is a field, an unknown vector index, or an abstract struct type
//!
//! Abstract addresses are a set containing constants and abstract access paths read from the environment. For example,
//! in the following Move code:
//!```ignore
//! struct S { f: u64 }
//!
//! fun foo(x: S) {
//!   let a = if (*) { 0x1 } else { *&x.f }
//!    ... // program point 1
//! }
//!```
//!, the value of `a` will be `{ 0x1, Footprint(x/f) }` at program point 1.

use crate::{
    dataflow_analysis::{AbstractDomain, SetDomain},
    function_target::FunctionTarget,
};
use move_core_types::language_storage::StructTag;
use move_model::{
    ast::TempIndex,
    model::{GlobalEnv, ModuleId, QualifiedId, StructId},
    ty::{PrimitiveType, Type, TypeDisplayContext},
};
use num::BigUint;
use std::{fmt, fmt::Formatter};

type Address = BigUint;

// =================================================================================================
// Data Model

/// Fully qualified type identifier `base` bound to type actuals `types`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AbsStructType {
    /// Module ID and struct ID
    base: QualifiedId<StructId>,
    /// Instantiation of generic type parameters
    types: Vec<Type>,
}

/// Building block for abstraction of addresses
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Addr {
    /// Account address constant
    Constant(Address),
    /// Account address read from given access path. This represents the value read from the given path at the beginning of
    /// the current function
    Footprint(AccessPath),
}

/// Abstraction of an address: non-empty set of constant or footprint address values
pub type AbsAddr = SetDomain<Addr>;

/// Abstraction of a key of type `addr`::`ty` in global storage
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalKey {
    /// Account address of key
    addr: AbsAddr,
    /// Type of key
    ty: AbsStructType,
}

/// Root of an access path: a global, local, or return variable
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Root {
    /// A key in global storage
    Global(GlobalKey), // TODO: this could (and maybe should) be AbsAddr + Offset::Global
    /// A local variable or formal parameter
    Local(TempIndex),
    /// A return variable
    Return(usize),
}

/// Offset of an access path: either a field, vector index, or global key
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Offset {
    /// Index into contents of a struct by field offset
    Field(usize),
    /// Unknown index into a vector
    VectorIndex,
    /// A type index into global storage. Only follows a field or vector index of type address
    Global(AbsStructType),
}

/// A unique identifier for a memory cell: root followed by zero or more offsets
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccessPath {
    root: Root,
    offsets: Vec<Offset>,
}

// =================================================================================================
// Abstract domain operations

/// Trait for a domain that can be viewed as a partial map from access paths to values
pub trait AccessPathMap<T: AbstractDomain> {
    fn get_access_path(&self, ap: AccessPath) -> Option<&T>;
}

/// Trait for an abstract domain that can represent footprint values
pub trait FootprintDomain: AbstractDomain {
    /// Create a footprint value for access path `ap`
    fn make_footprint(ap: AccessPath) -> Option<Self>;
}

impl Addr {
    /// Create a constant address from concrete address `a`
    pub fn constant(a: Address) -> Self {
        Addr::Constant(a)
    }

    /// Create a footprint address from access path `ap`
    pub fn footprint(ap: AccessPath) -> Self {
        Self::Footprint(ap)
    }

    /// Return `true` if `self` is a constant
    pub fn is_constant(&self) -> bool {
        match self {
            Self::Constant(_) => true,
            Self::Footprint(_) => false,
        }
    }

    /// Convert this address-typed abstract value A into an access path A/mid::sid::types
    pub fn add_struct_offset(self, mid: &ModuleId, sid: StructId, types: Vec<Type>) -> AccessPath {
        match self {
            Self::Footprint(mut ap) => {
                // TODO: assert type address?
                ap.add_offset(Offset::global(mid, sid, types));
                ap
            }
            Self::Constant(a) => AccessPath::new_address_constant(a, mid, sid, types),
        }
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionTarget) -> AddrDisplay<'a> {
        AddrDisplay { addr: self, env }
    }
}

impl AbsAddr {
    /// Create a constant address from concrete address `a`
    pub fn constant(a: Address) -> Self {
        SetDomain::singleton(Addr::Constant(a))
    }

    /// Create a footprint address from access path `ap`
    pub fn footprint(ap: AccessPath) -> Self {
        SetDomain::singleton(Addr::Footprint(ap))
    }

    /// Create a footprint address read from formal `temp_index`
    pub fn formal(formal_index: TempIndex) -> Self {
        SetDomain::footprint(AccessPath::new_local(formal_index))
    }

    /// Return `true` if `self` is a constant
    pub fn is_constant(&self) -> bool {
        self.0.iter().all(|a| a.is_constant())
    }

    /// Substitute all occurences of Footprint(ap) in `self` by resolving the accesss path
    /// `ap` in `sub_map`
    pub fn substitute_footprint(
        &mut self,
        actuals: &[AbsAddr],
        type_actuals: &[Type],
        sub_map: &dyn AccessPathMap<AbsAddr>,
    ) {
        let mut acc = SetDomain::default();
        for a in self.0.iter() {
            match a {
                Addr::Footprint(ap) => {
                    acc.join(&ap.substitute_footprint(actuals, type_actuals, sub_map));
                }
                c => {
                    acc.insert(c.clone());
                }
            }
        }
        *self = acc
    }

    /// Return a new abstract address by adding the offset `mid::sid<types>` to each element
    /// of `self`
    pub fn add_struct_offset(self, mid: &ModuleId, sid: StructId, types: Vec<Type>) -> Self {
        let mut acc = Self::default();
        for v in self.0.into_iter() {
            acc.insert(Addr::Footprint(v.add_struct_offset(
                mid,
                sid,
                types.clone(),
            )));
        }
        acc
    }

    /// Return a new abstract address by adding the offset `offset` to each element of `self`
    pub fn add_offset(&self, offset: Offset) -> Self {
        let mut extended_aps: AbsAddr = AbsAddr::default();
        for p in self.iter() {
            match p {
                Addr::Footprint(ap) => {
                    let mut extended_ap = ap.clone();
                    extended_ap.add_offset(offset.clone());
                    extended_aps.insert(Addr::Footprint(extended_ap));
                }
                Addr::Constant(_) => {
                    panic!("Type error: address constant as base")
                }
            }
        }
        extended_aps
    }

    /// Produce a new version of `self` with `prefix` prepended to each footprint
    /// value
    pub fn prepend(self, prefix: AccessPath) -> Self {
        let mut acc = Self::default();
        for v in self.0.into_iter() {
            match v {
                Addr::Footprint(ap) => {
                    let mut new_ap = ap.clone();
                    new_ap.prepend(prefix.clone());
                    acc.insert(Addr::Footprint(new_ap));
                }
                a => {
                    acc.insert(a);
                }
            }
        }
        acc
    }

    /// return an iterator over the footprint paths in `self`
    pub fn footprint_paths(&self) -> impl Iterator<Item = &AccessPath> {
        self.iter().filter_map(|a| match a {
            Addr::Footprint(ap) => Some(ap),
            Addr::Constant(_) => None,
        })
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionTarget) -> AbsAddrDisplay<'a> {
        AbsAddrDisplay { addr: self, env }
    }
}

impl FootprintDomain for AbsAddr {
    fn make_footprint(ap: AccessPath) -> Option<Self> {
        Some(AbsAddr::footprint(ap))
    }
}

impl GlobalKey {
    pub fn new(addr: AbsAddr, mid: &ModuleId, sid: StructId, types: Vec<Type>) -> Self {
        Self {
            addr,
            ty: AbsStructType::new(mid, sid, types),
        }
    }

    /// Create a constant `GlobalKey` using constant `addr` and type `ty`
    pub fn constant(addr: BigUint, ty: AbsStructType) -> Self {
        Self {
            addr: AbsAddr::constant(addr),
            ty,
        }
    }

    /// Return true if the address and type parameters of this global key are known statically
    pub fn is_statically_known(&self) -> bool {
        self.addr.is_constant() && self.ty.is_closed()
    }

    /// Substitute all occurences of Footprint(ap) in `self.addr` by resolving the accesss path
    /// `ap` in `sub_map`.
    pub fn substitute_footprint(
        &mut self,
        actuals: &[AbsAddr],
        type_actuals: &[Type],
        sub_map: &dyn AccessPathMap<AbsAddr>,
    ) {
        self.addr
            .substitute_footprint(actuals, type_actuals, sub_map);
        self.ty.substitute_footprint(type_actuals);
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionTarget) -> GlobalKeyDisplay<'a> {
        GlobalKeyDisplay { g: self, env }
    }
}

impl Root {
    /// Create a `Root` from local variable `index`
    pub fn local(index: TempIndex) -> Self {
        Root::Local(index)
    }

    /// Create a `Root` from global storage key `key`
    pub fn global(key: GlobalKey) -> Self {
        Root::Global(key)
    }

    /// Create a `Root` from return variable `index`
    pub fn ret(index: usize) -> Self {
        Root::Return(index)
    }

    /// Return the type of `self` in `fun`
    pub fn get_type(&self, fun: &FunctionTarget) -> Type {
        match self {
            Self::Global(g) => g.ty.get_type(),
            Self::Local(i) => fun.get_local_type(*i).clone(),
            Self::Return(i) => fun.get_return_type(*i).clone(),
        }
    }

    /// Return true if this variable is a formal parameter of `fun`
    pub fn is_formal(&self, fun: &FunctionTarget) -> bool {
        match self {
            Self::Local(i) => fun.func_env.is_parameter(*i),
            Self::Global(_) => false,
            Self::Return(_) => false,
        }
    }

    /// Replace all footprint paths in `self` using `actuals` and `sub_map`.
    /// Bind free type variables to `type_actuals`.
    pub fn substitute_footprint(
        &mut self,
        actuals: &[AbsAddr],
        type_actuals: &[Type],
        sub_map: &dyn AccessPathMap<AbsAddr>,
    ) {
        match self {
            Self::Global(g) => g.substitute_footprint(actuals, type_actuals, sub_map),
            Self::Local(_) | Self::Return(_) => (),
        }
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionTarget) -> RootDisplay<'a> {
        RootDisplay { root: self, env }
    }
}

impl Offset {
    pub fn global(mid: &ModuleId, sid: StructId, types: Vec<Type>) -> Self {
        Offset::Global(AbsStructType::new(mid, sid, types))
    }

    pub fn field(f: usize) -> Self {
        Offset::Field(f)
    }

    /// Get the type of offset `base`/`self` in function `fun`
    pub fn get_type(&self, base: &Type, env: &GlobalEnv) -> Type {
        match (base.skip_reference(), self) {
            (Type::Struct(mid, sid, types), Offset::Field(f)) => {
                let field_type = env
                    .get_module(*mid)
                    .get_struct(*sid)
                    .get_field_by_offset(*f)
                    .get_type();
                field_type.instantiate(types)
            }
            (Type::Vector(t), Offset::VectorIndex) => (*t.clone()),
            (Type::Primitive(PrimitiveType::Address), Offset::Global(s)) => s.get_type(),
            (Type::Primitive(PrimitiveType::Signer), Offset::Global(s)) => {
                // we conflate address and signer, so this can happen
                s.get_type()
            }
            _ => panic!(
                "Invalid base type {:?} for offset {:?} in get_type",
                base, self
            ),
        }
    }

    /// Bind free type variables in `self` to `type_actuals`
    pub fn substitute_footprint(&mut self, type_actuals: &[Type]) {
        match self {
            Offset::Global(g) => g.substitute_footprint(type_actuals),
            Offset::Field(..) | Offset::VectorIndex => (),
        }
    }

    /// Return true if this offset is the same in all concrete program executions
    pub fn is_statically_known(&self) -> bool {
        use Offset::*;
        match self {
            Field(..) => true,
            Global(..) | VectorIndex => false,
        }
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, base_type: &'a Type, env: &'a GlobalEnv) -> OffsetDisplay<'a> {
        OffsetDisplay {
            offset: self,
            base_type,
            env,
        }
    }
}

impl AccessPath {
    pub fn new(root: Root, offsets: Vec<Offset>) -> Self {
        AccessPath { root, offsets }
    }

    pub fn new_root(root: Root) -> Self {
        AccessPath {
            root,
            offsets: vec![],
        }
    }

    pub fn new_global(addr: AbsAddr, mid: &ModuleId, sid: StructId, types: Vec<Type>) -> Self {
        Self::new_root(Root::Global(GlobalKey::new(addr, mid, sid, types)))
    }

    pub fn new_address_constant(
        addr: BigUint,
        mid: &ModuleId,
        sid: StructId,
        types: Vec<Type>,
    ) -> Self {
        Self::new_global(AbsAddr::constant(addr), mid, sid, types)
    }

    pub fn new_global_constant(addr: BigUint, ty: AbsStructType) -> Self {
        Self::new_root(Root::Global(GlobalKey::constant(addr, ty)))
    }

    pub fn new_local(i: TempIndex) -> Self {
        Self::new_root(Root::Local(i))
    }

    /// Unpack `self` into its root and offsets
    pub fn into(self) -> (Root, Vec<Offset>) {
        (self.root, self.offsets)
    }

    pub fn root(&self) -> &Root {
        &self.root
    }

    pub fn offsets(&self) -> &[Offset] {
        &self.offsets
    }

    /// extend this access path by adding offset `o` to the end
    pub fn add_offset(&mut self, o: Offset) {
        self.offsets.push(o)
    }

    /// Return the type of this access path
    pub fn get_type(&self, fun: &FunctionTarget) -> Type {
        let mut ty = self.root.get_type(fun);
        for offset in &self.offsets {
            let offset_ty = offset.get_type(&ty, fun.module_env().env);
            ty = offset_ty;
        }
        ty
    }

    /// prepend `prefix` to self by swapping `self`'s root for prefix.root and
    /// replacing `self`'s accesses with prefix.accesses :: self.accesses
    pub fn prepend(&mut self, prefix: Self) {
        // TODO: assert root is a formal
        self.root = prefix.root;
        let mut suffix_offsets = self.offsets.clone();
        self.offsets = prefix.offsets;
        self.offsets.append(&mut suffix_offsets)
    }

    /// Construct a new abstract address by prepending the addresses in `addrs` to `self`
    pub fn prepend_addrs(&self, addrs: &AbsAddr) -> AbsAddr {
        let mut acc = AbsAddr::default();
        for a in addrs.iter() {
            match a {
                Addr::Footprint(ap) => {
                    let mut new_ap = self.clone();
                    new_ap.prepend(ap.clone());
                    acc.insert(Addr::footprint(new_ap));
                }
                Addr::Constant(c) => {
                    if self.offsets.is_empty() {
                        acc.insert(Addr::constant(c.clone()));
                    } else {
                        // access path with constant base and offsets (e.g., 0x1/M::S/f/g)
                        // normalize by converting into a path with a global base instead
                        match &self.offsets[0] {
                            Offset::Global(struct_type) => {
                                let root = Root::Global(GlobalKey::constant(
                                    c.clone(),
                                    struct_type.clone(),
                                ));
                                let mut new_offsets = vec![];
                                for v in self.offsets.iter().skip(0) {
                                    new_offsets.push(v.clone())
                                }
                                acc.insert(Addr::footprint(AccessPath::new(root, new_offsets)));
                            }
                            _ => panic!(
                                "Invariant violation: constant root with bad offsets {:?}",
                                self.offsets
                            ),
                        }
                    }
                }
            }
        }
        acc
    }

    /// Replace all footprint paths in `self` using `actuals` and `sub_map`.
    /// Bind free type variables to `type_actuals`.
    pub fn substitute_footprint(
        &self,
        actuals: &[AbsAddr],
        type_actuals: &[Type],
        sub_map: &dyn AccessPathMap<AbsAddr>,
    ) -> AbsAddr {
        let mut acc = AbsAddr::default();
        match &self.root {
            Root::Local(i) => {
                acc.join(&self.prepend_addrs(&actuals[*i]));
            }
            Root::Global(g) => {
                let mut new_g = g.clone();
                new_g.substitute_footprint(actuals, type_actuals, sub_map);
                let mut new_offsets = self.offsets.clone();
                new_offsets.iter_mut().for_each(|o| {
                    o.substitute_footprint(type_actuals);
                });
                acc.insert(Addr::footprint(AccessPath::new(
                    Root::Global(new_g),
                    new_offsets,
                )));
            }
            Root::Return(_) => (),
        }
        acc
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a FunctionTarget) -> AccessPathDisplay<'a> {
        AccessPathDisplay { ap: self, env }
    }
}

impl AbsStructType {
    pub fn new(mid: &ModuleId, sid: StructId, types: Vec<Type>) -> Self {
        AbsStructType {
            base: mid.qualified(sid),
            types,
        }
    }

    /// Return the concrete type of `self`
    pub fn get_type(&self) -> Type {
        Type::Struct(self.base.module_id, self.base.id, self.types.clone())
    }

    /// Substitue the open types in self.types with caller `type_actuals`
    pub fn substitute_footprint(&mut self, type_actuals: &[Type]) {
        for t in self.types.iter_mut() {
            *t = t.instantiate(type_actuals)
        }
    }

    /// Returns a normalized representation of this type if it closed,
    /// None if it is open
    pub fn normalize(&self, env: &GlobalEnv) -> Option<StructTag> {
        self.get_type().into_struct_tag(env)
    }

    /// Return `true` if `self` has no type variables or if all of `self`'s type variables are bound
    pub fn is_closed(&self) -> bool {
        self.types.iter().all(|t| t.is_open())
    }

    /// Return a wrapper of `self` that implements `Display` using `env`
    pub fn display<'a>(&'a self, env: &'a GlobalEnv) -> AbsStructTypeDisplay<'a> {
        AbsStructTypeDisplay { s: self, env }
    }
}

// =================================================================================================
// Formatting

pub struct AbsStructTypeDisplay<'a> {
    s: &'a AbsStructType,
    env: &'a GlobalEnv,
}

impl<'a> fmt::Display for AbsStructTypeDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.s.normalize(self.env) {
            Some(t) => {
                write!(f, "{}", t)
            }
            None => {
                let tctx = TypeDisplayContext::WithEnv {
                    env: self.env,
                    type_param_names: None,
                };
                let dummy_type =
                    Type::Struct(self.s.base.module_id, self.s.base.id, self.s.types.clone());
                write!(f, "{}", dummy_type.display(&tctx))
            }
        }
    }
}

pub struct AddrDisplay<'a> {
    addr: &'a Addr,
    env: &'a FunctionTarget<'a>,
}

impl<'a> fmt::Display for AddrDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.addr {
            Addr::Constant(a) => write!(f, "{:#x}", a),
            Addr::Footprint(ap) => write!(f, "{}", ap.display(self.env)),
        }
    }
}

pub struct AbsAddrDisplay<'a> {
    addr: &'a AbsAddr,
    env: &'a FunctionTarget<'a>,
}

impl<'a> fmt::Display for AbsAddrDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.addr.len() == 1 {
            write!(f, "{}", self.addr.iter().next().unwrap().display(self.env))
        } else {
            f.write_str("{")?;
            for a in self.addr.iter() {
                write!(f, "{}", a.display(self.env))?;
                // TODO: nice comma-separated list
                f.write_str(", ")?;
            }
            f.write_str("}")
        }
    }
}

pub struct GlobalKeyDisplay<'a> {
    g: &'a GlobalKey,
    env: &'a FunctionTarget<'a>,
}

impl<'a> fmt::Display for GlobalKeyDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}",
            self.g.addr.display(self.env),
            self.g.ty.display(self.env.module_env().env)
        )
    }
}

pub struct RootDisplay<'a> {
    root: &'a Root,
    env: &'a FunctionTarget<'a>,
}

impl<'a> fmt::Display for RootDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.root {
            Root::Global(g) => write!(f, "{}", g.display(self.env)),
            Root::Local(i) => write!(f, "Loc({})", i), // TODO: print name if available
            Root::Return(i) => write!(f, "Ret({})", i),
        }
    }
}

pub struct OffsetDisplay<'a> {
    offset: &'a Offset,
    base_type: &'a Type,
    env: &'a GlobalEnv,
}

impl<'a> fmt::Display for OffsetDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Offset::*;
        match self.offset {
            Field(fld) => match self.base_type.skip_reference() {
                Type::Struct(mid, sid, _types) => f.write_str(
                    self.env
                        .get_module(*mid)
                        .get_struct(*sid)
                        .get_field_by_offset(*fld)
                        .get_identifier()
                        .as_str(),
                ),
                _ => panic!(
                    "Invalid base type {:?} for field offset {:?}",
                    self.base_type, self.offset
                ),
            },
            VectorIndex => f.write_str("[_]"),
            Global(g) => write!(f, "{}", g.display(self.env)),
        }
    }
}

pub struct AccessPathDisplay<'a> {
    ap: &'a AccessPath,
    env: &'a FunctionTarget<'a>,
}

impl<'a> fmt::Display for AccessPathDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let genv = self.env.func_env.module_env.env;
        write!(f, "{}", self.ap.root.display(self.env))?;
        let mut root_ty = self.ap.root.get_type(self.env);
        for offset in &self.ap.offsets {
            f.write_str("/")?;
            write!(f, "{}", offset.display(&root_ty, genv))?;
            let offset_ty = offset.get_type(&root_ty, genv);
            root_ty = offset_ty;
        }
        Ok(())
    }
}
