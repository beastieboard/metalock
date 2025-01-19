
mod types;


use std::marker::PhantomData;

use anchor_lang::prelude::*;

use crate::{eval::{Evaluator, EvaluatorContext}, expr::Function, newval::{schema_is_superset, SchemaParser, SchemaType}, program::Program, schema::*, types::*};

use crate::encode::Encode;


#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MetalockTest {
    hooks: MetalockHooks,
    //resources: MetalockResources
}

impl HasMetalock for MetalockTest {
    fn get_hooks(&self) -> &MetalockHooks { &self.hooks }
    fn get_hooks_mut(&mut self) -> &mut MetalockHooks { &mut self.hooks }
    //fn get_resources(&self) -> &MetalockResources { &self.resources }
    //fn get_resources_mut(&mut self) -> &mut MetalockResources { &mut self.resources }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetalockHook {
    schema: Schema,
    name: String,
    code: Vec<u8>
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MetalockHooks(Vec<MetalockHook>);
impl_deref!([], MetalockHooks, Vec<MetalockHook>, 0);
pub type MetalockResources = Vec<(Schema, Vec<(String, ResourceDataOrPtr)>)>;

pub trait HasMetalock {
    fn get_hooks(&self) -> &MetalockHooks;
    fn get_hooks_mut(&mut self) -> &mut MetalockHooks;
    //fn get_resources(&self) -> &MetalockResources;
    //fn get_resources_mut(&mut self) -> &mut MetalockResources;
}

pub struct Metalock<'a, S: HasMetalock>(pub &'a mut S);

//impl<'a, S: HasMetalock> Metalock<'a, S> {
//    pub fn get_typed<T: SchemaType + Decode>(&self, name: String) -> Option<T> {
//        let typ = T::to_schema();
//        self._get(&ResourceId(typ, name)).map(
//            |b| Decode::rd_decode(&mut &**b).expect("Could not deserialize")
//        )
//    }
//    pub fn set_typed<T: SchemaType + Encode>(&mut self, name: String, data: T) {
//        let t = T::to_schema();
//        let d = data.rd_encode();
//        self._set(ResourceId(t, name), ResourceDataOrPtr::Data(d), Self::SET_INSERT);
//    }
//    pub fn set_with_schema(&mut self, schema: &Schema, name: String, data: RD) {
//        panic!("set_with_schema")
//        //assert!(validate_resource_data(&"", schema, &data).is_ok(), "Resources::set: invalid data");
//        //self._set(ResourceId(schema.encode(), name), data.into(), Self::SET_INSERT | Self::SET_UPDATE);
//    }
//
//    const SET_INSERT: u8 = 1;
//    const SET_UPDATE: u8 = 2;
//
//    fn _set(&mut self, ResourceId(typ, name): ResourceId, data: ResourceDataOrPtr, op: u8) -> SetResult {
//        if let Some((_, resources)) = self.0.get_resources_mut().iter_mut().find(|r| r.0 == typ) {
//            if let Some(v) = resources.iter_mut().find(|f| f.0 == name) {
//                if op & 1 > 0 {
//                    return SetResult::Replaced(std::mem::replace(&mut v.1, data));
//                }
//            }
//            if op & 2 > 0 {    
//                resources.push((name, data.into()));
//                return SetResult::Inserted;
//            }
//        } else {
//            if op & 2 > 0 {    
//                self.0.get_resources_mut().push((typ, vec![(name, data.into())]));
//                return SetResult::Inserted;
//            }
//        }
//        SetResult::Noop
//    }
//    //pub fn take(&mut self, ResourceId(typ, name): ResourceId) -> Option<ResourceData> {
//    //    if let Some(idxa) = self.get_resources().iter().position(|r| r.0 == typ) {
//    //        let d = &mut self.get_resources()[idxa].1;
//    //        if let Some(idxb) = d.iter().position(|f| f.0 == name) {
//    //            let o = Some(d.remove(idxb).1);
//    //            if d.len() == 0 {
//    //                self.get_resources().remove(idxa);
//    //            }
//    //            return o;
//    //        }
//    //    }
//    //    None
//    //}
//    //
//
//    /*
//     * Upgrade allows additional fields to be added to a schema,
//     * for example, if you have the schema:
//     *
//     * { members: [{ name: String, age: u8 }] }
//     *
//     * You can upgrade it to a compatible schema:
//     *   
//     * { members: [{ name: String, age: u8, balance: u64 }] }
//     *
//     * And the old key will point to the new key for backwards compatability
//     */
//    //pub fn upgrade(&mut self, old_id: ResourceId, new_id: ResourceId) {
//    //    assert!(
//    //        schema_is_superset(&old_id.0.to_schema(), &new_id.0.to_schema()),
//    //        "Resources::upgrade: not superset"
//    //    );
//
//    //    let ptr = ResourceDataOrPtr::Ptr(new_id.clone());
//    //    let old = self._set(old_id, ptr, Self::SET_UPDATE);
//    //    let old = match old {
//    //        SetResult::Replaced(d) => d,
//    //        _ => panic!("Resources::upgrade: old does not exist")
//    //    };
//
//    //    let r = self._set(new_id, old, Self::SET_INSERT);
//    //    assert!(r == SetResult::Inserted, "Resources::upgrade: new already exists");
//    //}
//
//    fn _get(&self, id: &ResourceId) -> Option<&Vec<u8>> {
//        let r = &self.0.get_resources().iter().find(|r| r.0 == id.0)?.1.iter().find(|f| f.0 == id.1)?.1;
//        match r {
//            ResourceDataOrPtr::Data(d) => Some(d),
//            ResourceDataOrPtr::Ptr(p) => self._get(p)
//        }
//    }
//}



#[derive(Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct ResourceId(pub Schema, pub String);

#[derive(Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub enum ResourceDataOrPtr {
    Data(Vec<u8>),
    Ptr(ResourceId)
}

impl_into!([], ResourceDataOrPtr, ResourceData, |self| ResourceDataOrPtr::Data(self.rd_encode()));
impl_into!([], ResourceDataOrPtr, ResourceId, |self| ResourceDataOrPtr::Ptr(self));

#[derive(PartialEq, Eq)]
enum SetResult {
    Replaced(ResourceDataOrPtr),
    Inserted,
    Noop
}




use crate::schema::tag::{self, TagType};

/*
 * Hooks
 */

impl<'a, S: HasMetalock> Metalock<'a, S> {
    pub fn add_hook(&mut self, name: String, bin: Vec<u8>) -> Result<()> {

        // take schema from bin
        let parser = ParserBuffer::new(&bin);
        let mut parser = SchemaParser(parser);
        assert!(parser[0] == tag::FUNCTION::ID);
        let schema = parser.take_schema();
        let hook = MetalockHook {
            name,
            schema,
            code: parser.to_vec()
        };

        let r = self.0.get_hooks().binary_search(&hook);
        match r {
            Ok(_) => panic!("Hook exists"),
            Err(idx) => {
                self.0.get_hooks_mut().insert(idx, hook);
                Ok(())
            }
        }
    }

    pub fn call_hook_with_results<In: SchemaType + Into<RD>, Out: SchemaType>(
        &self,
        context: EvaluatorContext,
        name: String,
        input: &In,
        out_p: PhantomData<Out>
    ) -> Vec<std::result::Result<RD, String>> {

        let schema = Function::<In, Out>::to_schema();
        
        let needle = MetalockHook {
            schema: schema.clone(),
            name: name.clone(),
            code: Default::default()
        };
        let idx = self.0.get_hooks().binary_search(&needle).unwrap_or_else(|e| e);

        let mut results = vec![];

        for hook in &self.0.get_hooks()[idx..] {
            if hook.schema != schema || hook.name != name {
                break;
            } else {
                let mut eval = Evaluator::new(&mut hook.code.as_ref(), context.clone());
                let r = eval.run(input.clone().into());
                results.push(Ok(r));
            }
        }

        results
    }
}
